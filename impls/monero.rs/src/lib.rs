use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::{c_int, c_void};
use std::ptr::NonNull;
use std::sync::Arc;

pub mod bindings;
pub use bindings::WalletStatus_Critical;
pub use bindings::WalletStatus_Error;
pub use bindings::WalletStatus_Ok;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkType {
    Mainnet = bindings::NetworkType_MAINNET as isize,
    Testnet = bindings::NetworkType_TESTNET as isize,
    Stagenet = bindings::NetworkType_STAGENET as isize,
}

impl NetworkType {
    pub fn from_c_int(value: c_int) -> Option<Self> {
        match value {
            bindings::NetworkType_MAINNET => Some(NetworkType::Mainnet),
            bindings::NetworkType_TESTNET => Some(NetworkType::Testnet),
            bindings::NetworkType_STAGENET => Some(NetworkType::Stagenet),
            _ => None,
        }
    }

    pub fn to_c_int(self) -> c_int {
        self as c_int
    }
}

#[derive(Debug)]
pub enum WalletError {
    NullPointer,
    FfiError(String),
    WalletErrorCode(c_int, String),
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletError::NullPointer => write!(f, "null pointer from FFI"),
            WalletError::FfiError(msg) => write!(f, "FFI error: {}", msg),
            WalletError::WalletErrorCode(code, msg) => {
                write!(f, "wallet error (status {}): {}", code, msg)
            }
        }
    }
}

impl std::error::Error for WalletError {}

pub type WalletResult<T> = Result<T, WalletError>;

#[derive(Debug)]
pub struct Account {
    pub index: u32,
    pub label: String,
    pub balance: u64,
    pub unlocked_balance: u64,
}

#[derive(Debug)]
pub struct GetAccounts {
    pub accounts: Vec<Account>,
}

pub struct Wallet {
    pub ptr: NonNull<c_void>,
    pub manager: Arc<WalletManager>,
    pub is_closed: bool,
}

pub struct WalletManager {
    ptr: NonNull<c_void>,
}

#[derive(Debug, Clone)]
pub struct WalletConfig {
    pub daemon_address: String,
    pub upper_transaction_size_limit: u64,
    pub daemon_username: String,
    pub daemon_password: String,
    pub use_ssl: bool,
    pub light_wallet: bool,
    pub proxy_address: String,
}

impl Default for WalletConfig {
    fn default() -> Self {
        WalletConfig {
            daemon_address: "localhost:18081".to_string(),
            upper_transaction_size_limit: 10000, // TODO: set sane value.
            daemon_username: "".to_string(),
            daemon_password: "".to_string(),
            use_ssl: false,
            light_wallet: false,
            proxy_address: "".to_string(),
        }
    }
}

pub type BlockHeight = u64;

#[derive(Debug)]
pub struct Refreshed;

#[derive(Debug, Clone)]
pub struct Destination {
    pub address: String,
    /// In atomic units.
    pub amount: u64,
}

#[derive(Debug)]
pub struct Transfer {
    pub txid: String,
    pub tx_key: Option<String>,
    pub amount: u64,
    pub fee: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckTxKey {
    pub valid: bool,
    pub error: Option<String>,
}

impl WalletManager {
    pub fn new() -> WalletResult<Arc<Self>> {
        unsafe {
            bindings::MONERO_WalletManagerFactory_setLogLevel(4);
            let ptr = bindings::MONERO_WalletManagerFactory_getWalletManager();
            let ptr = NonNull::new(ptr).ok_or(WalletError::NullPointer)?;
            Ok(Arc::new(WalletManager { ptr }))
        }
    }

    pub fn get_status(&self, wallet_ptr: *mut c_void) -> WalletResult<()> {
        if wallet_ptr.is_null() {
            return Err(WalletError::NullPointer);
        }

        unsafe {
            let status = bindings::MONERO_Wallet_status(wallet_ptr);

            if status == bindings::WalletStatus_Ok {
                Ok(())
            } else {
                let error_ptr = bindings::MONERO_Wallet_errorString(wallet_ptr);
                let error_msg = if error_ptr.is_null() {
                    "Unknown error".to_string()
                } else {
                    CStr::from_ptr(error_ptr).to_string_lossy().into_owned()
                };
                Err(WalletError::WalletErrorCode(status, error_msg))
            }
        }
    }

    pub fn throw_if_error(&self, wallet_ptr: *mut c_void) -> WalletResult<()> {
        self.get_status(wallet_ptr)
    }

    pub fn create_wallet(
        self: &Arc<Self>,
        path: &str,
        password: &str,
        language: &str,
        network_type: NetworkType,
    ) -> WalletResult<Wallet> {
        let c_path =
            CString::new(path).map_err(|_| WalletError::FfiError("Invalid path".to_string()))?;
        let c_password = CString::new(password)
            .map_err(|_| WalletError::FfiError("Invalid password".to_string()))?;
        let c_language = CString::new(language)
            .map_err(|_| WalletError::FfiError("Invalid language".to_string()))?;

        unsafe {
            let wallet_ptr = bindings::MONERO_WalletManager_createWallet(
                self.ptr.as_ptr(),
                c_path.as_ptr(),
                c_password.as_ptr(),
                c_language.as_ptr(),
                network_type.to_c_int(),
            );

            self.throw_if_error(wallet_ptr)?;
            if wallet_ptr.is_null() {
                return Err(WalletError::NullPointer);
            }

            Ok(Wallet {
                ptr: NonNull::new(wallet_ptr).unwrap(),
                manager: Arc::clone(self),
                is_closed: false,
            })
        }
    }

    pub fn restore_mnemonic(
        self: &Arc<Self>,
        path: String,
        password: String,
        seed: String,
        network_type: NetworkType,
        restore_height: u64,
        kdf_rounds: u64,
        seed_offset: String,
    ) -> WalletResult<Wallet> {
        let c_path = CString::new(path)
            .map_err(|_| WalletError::FfiError("Invalid path string".to_string()))?;
        let c_password = CString::new(password)
            .map_err(|_| WalletError::FfiError("Invalid password string".to_string()))?;
        let c_seed = CString::new(seed)
            .map_err(|_| WalletError::FfiError("Invalid seed string".to_string()))?;
        let c_seed_offset = CString::new(seed_offset)
            .map_err(|_| WalletError::FfiError("Invalid seed_offset string".to_string()))?;

        unsafe {
            let wallet_ptr = bindings::MONERO_WalletManager_recoveryWallet(
                self.ptr.as_ptr(),
                c_path.as_ptr(),
                c_password.as_ptr(),
                c_seed.as_ptr(),
                network_type.to_c_int(),
                restore_height,
                kdf_rounds,
                c_seed_offset.as_ptr(),
            );

            self.throw_if_error(wallet_ptr)?;
            if wallet_ptr.is_null() {
                return Err(WalletError::NullPointer);
            }

            Ok(Wallet {
                ptr: NonNull::new(wallet_ptr).unwrap(),
                manager: Arc::clone(self),
                is_closed: false,
            })
        }
    }

    pub fn restore_polyseed(
        self: &Arc<Self>,
        path: String,
        password: String,
        polyseed: String,
        network_type: NetworkType,
        restore_height: u64,
        kdf_rounds: u64,
        seed_offset: String,
        new_wallet: bool,
    ) -> WalletResult<Wallet> {
        let c_path = CString::new(path)
            .map_err(|_| WalletError::FfiError("Invalid path string".to_string()))?;
        let c_password = CString::new(password)
            .map_err(|_| WalletError::FfiError("Invalid password string".to_string()))?;
        let c_polyseed = CString::new(polyseed)
            .map_err(|_| WalletError::FfiError("Invalid mnemonic string".to_string()))?;
        let c_seed_offset = CString::new(seed_offset)
            .map_err(|_| WalletError::FfiError("Invalid seed offset string".to_string()))?;

        unsafe {
            let wallet_ptr = bindings::MONERO_WalletManager_createWalletFromPolyseed(
                self.ptr.as_ptr(),
                c_path.as_ptr(),
                c_password.as_ptr(),
                network_type.to_c_int(),
                c_polyseed.as_ptr(),
                c_seed_offset.as_ptr(),
                new_wallet,
                restore_height,
                kdf_rounds,
            );

            self.throw_if_error(wallet_ptr)?;
            if wallet_ptr.is_null() {
                return Err(WalletError::NullPointer);
            }

            Ok(Wallet {
                ptr: NonNull::new(wallet_ptr).unwrap(),
                manager: Arc::clone(self),
                is_closed: false,
            })
        }
    }

    pub fn generate_from_keys(
        self: &Arc<Self>,
        filename: String,
        address: String,
        spendkey: String,
        viewkey: String,
        restore_height: u64,
        password: String,
        language: String,
        network_type: NetworkType,
        kdf_rounds: u64,
    ) -> WalletResult<Wallet> {
        let c_filename = CString::new(filename)
            .map_err(|_| WalletError::FfiError("Invalid filename".to_string()))?;
        let c_password = CString::new(password)
            .map_err(|_| WalletError::FfiError("Invalid password".to_string()))?;
        let c_language = CString::new(language)
            .map_err(|_| WalletError::FfiError("Invalid language".to_string()))?;
        let c_address = CString::new(address)
            .map_err(|_| WalletError::FfiError("Invalid address".to_string()))?;
        let c_spendkey = CString::new(spendkey)
            .map_err(|_| WalletError::FfiError("Invalid spendkey".to_string()))?;
        let c_viewkey = CString::new(viewkey)
            .map_err(|_| WalletError::FfiError("Invalid viewkey".to_string()))?;

        unsafe {
            let wallet_ptr = bindings::MONERO_WalletManager_createWalletFromKeys(
                self.ptr.as_ptr(),
                c_filename.as_ptr(),
                c_password.as_ptr(),
                c_language.as_ptr(),
                network_type.to_c_int(),
                restore_height,
                c_address.as_ptr(),
                c_viewkey.as_ptr(),
                c_spendkey.as_ptr(),
                kdf_rounds,
            );

            if wallet_ptr.is_null() {
                return Err(WalletError::NullPointer);
            }

            self.throw_if_error(wallet_ptr)?;

            Ok(Wallet {
                ptr: NonNull::new(wallet_ptr).unwrap(),
                manager: Arc::clone(self),
                is_closed: false,
            })
        }
    }

    pub fn open_wallet(
        self: &Arc<Self>,
        path: &str,
        password: &str,
        network_type: NetworkType,
    ) -> WalletResult<Wallet> {
        let c_path =
            CString::new(path).map_err(|_| WalletError::FfiError("Invalid path".to_string()))?;
        let c_password = CString::new(password)
            .map_err(|_| WalletError::FfiError("Invalid password".to_string()))?;

        unsafe {
            let wallet_ptr = bindings::MONERO_WalletManager_openWallet(
                self.ptr.as_ptr(),
                c_path.as_ptr(),
                c_password.as_ptr(),
                network_type.to_c_int(),
            );

            if wallet_ptr.is_null() {
                return Err(WalletError::NullPointer);
            }

            self.throw_if_error(wallet_ptr)?;

            Ok(Wallet {
                ptr: NonNull::new(wallet_ptr).unwrap(),
                manager: Arc::clone(self),
                is_closed: false,
            })
        }
    }

    pub fn get_height(&self) -> WalletResult<BlockHeight> {
        unsafe {
            let height = bindings::MONERO_WalletManager_blockchainHeight(self.ptr.as_ptr());
            Ok(height)
        }
    }

    pub fn set_daemon_address(&self, daemon_address: &str) -> WalletResult<()> {
        let c_daemon_address = CString::new(daemon_address)
            .map_err(|_| WalletError::FfiError("Invalid daemon address".to_string()))?;

        unsafe {
            bindings::MONERO_WalletManager_setDaemonAddress(
                self.ptr.as_ptr(),
                c_daemon_address.as_ptr(),
            );
            Ok(())
        }
    }
}

impl Wallet {
    pub fn get_seed(&self, seed_offset: Option<&str>) -> WalletResult<String> {
        let c_seed_offset = CString::new(seed_offset.unwrap_or(""))
            .map_err(|_| WalletError::FfiError("Invalid seed_offset".to_string()))?;

        unsafe {
            let seed_ptr = bindings::MONERO_Wallet_seed(self.ptr.as_ptr(), c_seed_offset.as_ptr());

            self.throw_if_error()?;
            if seed_ptr.is_null() {
                return Err(self.get_last_error());
            }

            let seed = CStr::from_ptr(seed_ptr).to_string_lossy().into_owned();
            if seed.is_empty() {
                return Err(WalletError::FfiError("Received empty seed".to_string()));
            }

            Ok(seed)
        }
    }

    pub fn get_address(&self, account_index: u64, address_index: u64) -> WalletResult<String> {
        unsafe {
            let address_ptr =
                bindings::MONERO_Wallet_address(self.ptr.as_ptr(), account_index, address_index);

            self.throw_if_error()?;
            if address_ptr.is_null() {
                Err(self.get_last_error())
            } else {
                let address = CStr::from_ptr(address_ptr).to_string_lossy().into_owned();
                Ok(address)
            }
        }
    }

    pub fn is_deterministic(&self) -> WalletResult<bool> {
        unsafe {
            let result = bindings::MONERO_Wallet_isDeterministic(self.ptr.as_ptr());

            self.throw_if_error()?;
            Ok(result)
        }
    }

    pub fn get_last_error(&self) -> WalletError {
        unsafe {
            let error_ptr = bindings::MONERO_Wallet_errorString(self.ptr.as_ptr());
            let status = bindings::MONERO_Wallet_status(self.ptr.as_ptr());

            let error_msg = if error_ptr.is_null() {
                "Unknown error".to_string()
            } else {
                CStr::from_ptr(error_ptr).to_string_lossy().into_owned()
            };

            WalletError::WalletErrorCode(status, error_msg)
        }
    }

    pub fn throw_if_error(&self) -> WalletResult<()> {
        self.manager.get_status(self.ptr.as_ptr())
    }

    pub fn get_balance(&self, account_index: u32) -> WalletResult<GetBalance> {
        unsafe {
            let balance = bindings::MONERO_Wallet_balance(self.ptr.as_ptr(), account_index);
            self.throw_if_error()?;
            let unlocked_balance =
                bindings::MONERO_Wallet_unlockedBalance(self.ptr.as_ptr(), account_index);
            self.throw_if_error()?;
            Ok(GetBalance {
                balance,
                unlocked_balance,
            })
        }
    }

    pub fn create_account(&self, label: &str) -> WalletResult<()> {
        let c_label =
            CString::new(label).map_err(|_| WalletError::FfiError("Invalid label".to_string()))?;

        unsafe {
            bindings::MONERO_Wallet_addSubaddressAccount(self.ptr.as_ptr(), c_label.as_ptr());
            self.throw_if_error()
        }
    }

    pub fn get_accounts(&self) -> WalletResult<GetAccounts> {
        unsafe {
            let accounts_size = bindings::MONERO_Wallet_numSubaddressAccounts(self.ptr.as_ptr());
            self.throw_if_error()?;

            let mut accounts = Vec::new();

            for i in 0..accounts_size as u32 {
                let label_ptr = bindings::MONERO_Wallet_getSubaddressLabel(self.ptr.as_ptr(), i, 0);
                let label = if label_ptr.is_null() {
                    "Unnamed".to_string()
                } else {
                    CStr::from_ptr(label_ptr).to_string_lossy().into_owned()
                };

                let balance = bindings::MONERO_Wallet_balance(self.ptr.as_ptr(), i);
                let unlocked_balance =
                    bindings::MONERO_Wallet_unlockedBalance(self.ptr.as_ptr(), i);

                accounts.push(Account {
                    index: i,
                    label,
                    balance,
                    unlocked_balance,
                });
            }

            Ok(GetAccounts { accounts })
        }
    }

    /// Also called by Drop. Safe to call multiple times.
    pub fn close_wallet(&mut self) -> WalletResult<()> {
        if self.is_closed {
            return Ok(());
        }
        unsafe {
            let result = bindings::MONERO_WalletManager_closeWallet(
                self.manager.ptr.as_ptr(),
                self.ptr.as_ptr(),
                false,
            );
            if result {
                self.is_closed = true;
                Ok(())
            } else {
                Err(WalletError::FfiError("Failed to close wallet".to_string()))
            }
        }
    }

    /// Must be called after create/open before refresh.
    pub fn init(&self, config: WalletConfig) -> WalletResult<()> {
        let c_daemon_address = CString::new(config.daemon_address)
            .map_err(|_| WalletError::FfiError("Invalid daemon address".to_string()))?;
        let c_daemon_username = CString::new(config.daemon_username)
            .map_err(|_| WalletError::FfiError("Invalid daemon username".to_string()))?;
        let c_daemon_password = CString::new(config.daemon_password)
            .map_err(|_| WalletError::FfiError("Invalid daemon password".to_string()))?;
        let c_proxy_address = CString::new(config.proxy_address)
            .map_err(|_| WalletError::FfiError("Invalid proxy address".to_string()))?;

        unsafe {
            // Disable file logging.
            let c_empty = CString::new("").unwrap();
            let c_log_tag = CString::new("moneroc").unwrap();
            bindings::MONERO_Wallet_init3(
                self.ptr.as_ptr(),
                c_empty.as_ptr(),
                c_log_tag.as_ptr(),
                c_empty.as_ptr(),
                true,
            );

            let result = bindings::MONERO_Wallet_init(
                self.ptr.as_ptr(),
                c_daemon_address.as_ptr(),
                config.upper_transaction_size_limit,
                c_daemon_username.as_ptr(),
                c_daemon_password.as_ptr(),
                config.use_ssl,
                config.light_wallet,
                c_proxy_address.as_ptr(),
            );

            if result {
                Ok(())
            } else {
                Err(self.get_last_error())
            }
        }
    }

    pub fn refresh(&self) -> WalletResult<Refreshed> {
        unsafe {
            let result = bindings::MONERO_Wallet_refresh(self.ptr.as_ptr());
            if result {
                Ok(Refreshed)
            } else {
                Err(self.get_last_error())
            }
        }
    }

    pub fn refresh_async(&self) -> WalletResult<Refreshed> {
        unsafe {
            bindings::MONERO_Wallet_refreshAsync(self.ptr.as_ptr());
            Ok(Refreshed)
        }
    }

    pub fn transfer(
        &self,
        account_index: u32,
        destinations: Vec<Destination>,
        get_tx_key: bool,
        sweep_all: bool,
    ) -> WalletResult<Transfer> {
        let separator = ";";
        let separator_c = CString::new(separator)
            .map_err(|_| WalletError::FfiError("Invalid separator".to_string()))?;

        let addresses: Vec<String> = destinations.iter().map(|d| d.address.clone()).collect();
        let c_address_list = CString::new(addresses.join(separator))
            .map_err(|_| WalletError::FfiError("Invalid address list".to_string()))?;

        let amounts: Vec<String> = destinations.iter().map(|d| d.amount.to_string()).collect();
        let c_amount_list = CString::new(amounts.join(separator))
            .map_err(|_| WalletError::FfiError("Invalid amount list".to_string()))?;

        // TODO: payment IDs, preferred inputs.
        let payment_id = CString::new("").unwrap();
        let c_preferred_inputs = CString::new("").unwrap();
        let preferred_inputs_separator = CString::new("").unwrap();
        let mixin_count = 16;

        unsafe {
            let tx_ptr = bindings::MONERO_Wallet_createTransactionMultDest(
                self.ptr.as_ptr(),
                c_address_list.as_ptr(),
                separator_c.as_ptr(),
                payment_id.as_ptr(),
                sweep_all,
                c_amount_list.as_ptr(),
                separator_c.as_ptr(),
                mixin_count,
                bindings::Priority_Default,
                account_index,
                c_preferred_inputs.as_ptr(),
                preferred_inputs_separator.as_ptr(),
            );

            self.throw_if_error()?;
            if tx_ptr.is_null() {
                return Err(WalletError::NullPointer);
            }

            let tx_status = bindings::MONERO_PendingTransaction_status(tx_ptr);
            if tx_status != bindings::PendingTransactionStatus_Ok {
                let err_ptr = bindings::MONERO_PendingTransaction_errorString(tx_ptr);
                let err_msg = if err_ptr.is_null() {
                    "Unknown transaction error".to_string()
                } else {
                    CStr::from_ptr(err_ptr).to_string_lossy().into_owned()
                };
                return Err(WalletError::FfiError(err_msg));
            }

            let txid_ptr = bindings::MONERO_PendingTransaction_txid(tx_ptr, separator_c.as_ptr());
            if txid_ptr.is_null() {
                return Err(WalletError::FfiError(
                    "Failed to get transaction ID".to_string(),
                ));
            }
            let txid = CStr::from_ptr(txid_ptr).to_string_lossy().into_owned();
            let fee = bindings::MONERO_PendingTransaction_fee(tx_ptr);
            let amount = bindings::MONERO_PendingTransaction_amount(tx_ptr);

            let tx_key = if get_tx_key {
                let c_txid = CString::new(txid.clone())
                    .map_err(|_| WalletError::FfiError("Invalid txid".to_string()))?;
                let tx_key_ptr =
                    bindings::MONERO_Wallet_getTxKey(self.ptr.as_ptr(), c_txid.as_ptr());
                if tx_key_ptr.is_null() {
                    None
                } else {
                    Some(CStr::from_ptr(tx_key_ptr).to_string_lossy().into_owned())
                }
            } else {
                None
            };

            let empty_filename = CString::new("").unwrap();
            let commit_result =
                bindings::MONERO_PendingTransaction_commit(tx_ptr, empty_filename.as_ptr(), false);
            if !commit_result {
                let err_ptr = bindings::MONERO_PendingTransaction_errorString(tx_ptr);
                let err_msg = if err_ptr.is_null() {
                    "Failed to commit transaction".to_string()
                } else {
                    CStr::from_ptr(err_ptr).to_string_lossy().into_owned()
                };
                return Err(WalletError::FfiError(err_msg));
            }

            Ok(Transfer {
                txid,
                tx_key,
                amount,
                fee,
            })
        }
    }

    pub fn sweep_all(
        &self,
        account_index: u32,
        destination: Destination,
        get_tx_key: bool,
    ) -> WalletResult<Transfer> {
        let c_address = CString::new(destination.address.clone())
            .map_err(|_| WalletError::FfiError("Invalid address".to_string()))?;
        let empty = CString::new("").unwrap();
        let payment_id = CString::new("").unwrap();
        let c_preferred_inputs = CString::new("").unwrap();
        let preferred_inputs_separator = CString::new("").unwrap();
        let mixin_count = 16;

        unsafe {
            let tx_ptr = bindings::MONERO_Wallet_createTransactionMultDest(
                self.ptr.as_ptr(),
                c_address.as_ptr(),
                empty.as_ptr(),
                payment_id.as_ptr(),
                true,
                empty.as_ptr(),
                empty.as_ptr(),
                mixin_count,
                bindings::Priority_Default,
                account_index,
                c_preferred_inputs.as_ptr(),
                preferred_inputs_separator.as_ptr(),
            );

            self.throw_if_error()?;
            if tx_ptr.is_null() {
                return Err(WalletError::NullPointer);
            }

            let tx_status = bindings::MONERO_PendingTransaction_status(tx_ptr);
            if tx_status != bindings::PendingTransactionStatus_Ok {
                let err_ptr = bindings::MONERO_PendingTransaction_errorString(tx_ptr);
                let err_msg = if err_ptr.is_null() {
                    "Unknown transaction error".to_string()
                } else {
                    CStr::from_ptr(err_ptr).to_string_lossy().into_owned()
                };
                return Err(WalletError::FfiError(err_msg));
            }

            let txid_ptr = bindings::MONERO_PendingTransaction_txid(tx_ptr, empty.as_ptr());
            if txid_ptr.is_null() {
                return Err(WalletError::FfiError(
                    "Failed to get transaction ID".to_string(),
                ));
            }
            let txid = CStr::from_ptr(txid_ptr).to_string_lossy().into_owned();
            let fee = bindings::MONERO_PendingTransaction_fee(tx_ptr);
            let amount = bindings::MONERO_PendingTransaction_amount(tx_ptr);

            let tx_key = if get_tx_key {
                let c_txid = CString::new(txid.clone())
                    .map_err(|_| WalletError::FfiError("Invalid txid".to_string()))?;
                let tx_key_ptr =
                    bindings::MONERO_Wallet_getTxKey(self.ptr.as_ptr(), c_txid.as_ptr());
                if tx_key_ptr.is_null() {
                    None
                } else {
                    Some(CStr::from_ptr(tx_key_ptr).to_string_lossy().into_owned())
                }
            } else {
                None
            };

            let empty_filename = CString::new("").unwrap();
            let commit_result =
                bindings::MONERO_PendingTransaction_commit(tx_ptr, empty_filename.as_ptr(), false);
            if !commit_result {
                let err_ptr = bindings::MONERO_PendingTransaction_errorString(tx_ptr);
                let err_msg = if err_ptr.is_null() {
                    "Failed to commit sweep transaction".to_string()
                } else {
                    CStr::from_ptr(err_ptr).to_string_lossy().into_owned()
                };
                return Err(WalletError::FfiError(err_msg));
            }

            Ok(Transfer {
                txid,
                tx_key,
                amount,
                fee,
            })
        }
    }

    pub fn set_seed_language(&self, language: &str) -> WalletResult<()> {
        let c_language = CString::new(language)
            .map_err(|_| WalletError::FfiError("Invalid language string".to_string()))?;

        unsafe {
            bindings::MONERO_Wallet_setSeedLanguage(self.ptr.as_ptr(), c_language.as_ptr());
            self.throw_if_error()
        }
    }

    /// Note: the C API currently passes received/in_pool/confirmations by value,
    /// not by pointer, so output values cannot be retrieved.
    pub fn check_tx_key(
        &self,
        txid: String,
        tx_key: String,
        address: String,
        received: Option<u64>,
        in_pool: Option<bool>,
        confirmations: Option<u64>,
    ) -> WalletResult<CheckTxKey> {
        let c_txid = CString::new(txid)
            .map_err(|_| WalletError::FfiError("Invalid txid string".to_string()))?;
        let c_tx_key = CString::new(tx_key)
            .map_err(|_| WalletError::FfiError("Invalid tx_key string".to_string()))?;
        let c_address = CString::new(address)
            .map_err(|_| WalletError::FfiError("Invalid address string".to_string()))?;

        let result = unsafe {
            bindings::MONERO_Wallet_checkTxKey(
                self.ptr.as_ptr(),
                c_txid.as_ptr(),
                c_tx_key.as_ptr(),
                c_address.as_ptr(),
                received.unwrap_or(0),
                in_pool.unwrap_or(false),
                confirmations.unwrap_or(0),
            )
        };

        if result {
            Ok(CheckTxKey {
                valid: true,
                error: None,
            })
        } else {
            Err(WalletError::FfiError(
                "Transaction key is invalid.".to_string(),
            ))
        }
    }
}

#[derive(Debug)]
pub struct GetBalance {
    pub balance: u64,
    pub unlocked_balance: u64,
}

impl Drop for Wallet {
    fn drop(&mut self) {
        if !self.is_closed {
            let _ = self.close_wallet();
        }
    }
}
