use monero_c_rust::{
    NetworkType, WalletConfig, WalletError, WalletManager, WalletResult, WalletStatus_Ok,
};
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

const TEST_WALLET_NAMES: &[&str] = &[
    "test_wallet",
    "mainnet_wallet",
    "testnet_wallet",
    "stagenet_wallet",
];

fn cleanup_wallets(temp_dir: &TempDir) {
    for name in TEST_WALLET_NAMES {
        let _ = fs::remove_file(temp_dir.path().join(name));
        let _ = fs::remove_file(temp_dir.path().join(format!("{}.keys", name)));
        let _ = fs::remove_file(temp_dir.path().join(format!("{}.address.txt", name)));
    }
}

fn setup() -> WalletResult<(Arc<WalletManager>, TempDir)> {
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    cleanup_wallets(&temp_dir);
    let manager = WalletManager::new()?;
    Ok((manager, temp_dir))
}

#[test]
fn test_wallet_manager_creation() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let wallet_result =
        manager.create_wallet(wallet_str, "password", "English", NetworkType::Mainnet);
    assert!(wallet_result.is_ok(), "WalletManager creation failed");
}

#[test]
fn test_wallet_creation() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let wallet =
        manager.create_wallet(wallet_str, "password", "English", NetworkType::Mainnet).unwrap();
    assert!(wallet.is_deterministic().unwrap());
}

#[test]
fn test_restore_mnemonic() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap().to_string();
    let mnemonic = "hemlock jubilee eden hacksaw boil superior inroads epoxy exhale orders cavernous second brunt saved richly lower upgrade hitched launching deepest mostly playful layout lower eden".to_string();

    let wallet = manager
        .restore_mnemonic(
            wallet_str,
            "password".to_string(),
            mnemonic,
            NetworkType::Mainnet,
            0,
            1,
            "".to_string(),
        )
        .expect("Failed to restore wallet");

    assert!(wallet.is_deterministic().unwrap());
    assert_eq!(
        wallet.get_address(0, 0).unwrap(),
        "45wsWad9EwZgF3VpxQumrUCRaEtdyyh6NG8sVD3YRVVJbK1jkpJ3zq8WHLijVzodQ22LxwkdWx7fS2a6JzaRGzkNU8K2Dhi"
    );
}

#[test]
fn test_restore_polyseed() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap().to_string();
    let polyseed = "capital chief route liar question fix clutch water outside pave hamster occur always learn license knife".to_string();

    let wallet = manager
        .restore_polyseed(
            wallet_str,
            "password".to_string(),
            polyseed,
            NetworkType::Mainnet,
            0,
            1,
            "".to_string(),
            true,
        )
        .expect("Failed to restore wallet from polyseed");

    assert!(wallet.is_deterministic().unwrap());
    assert_eq!(
        wallet.get_address(0, 0).unwrap(),
        "465cUW8wTMSCV8oVVh7CuWWHs7yeB1oxhNPrsEM5FKSqadTXmobLqsNEtRnyGsbN1rbDuBtWdtxtXhTJda1Lm9vcH2ZdrD1"
    );
}

#[test]
fn test_generate_from_keys() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let wallet = manager
        .generate_from_keys(
            wallet_str.to_string(),
            "45wsWad9EwZgF3VpxQumrUCRaEtdyyh6NG8sVD3YRVVJbK1jkpJ3zq8WHLijVzodQ22LxwkdWx7fS2a6JzaRGzkNU8K2Dhi".to_string(),
            "29adefc8f67515b4b4bf48031780ab9d071d24f8a674b879ce7f245c37523807".to_string(),
            "3bc0b202cde92fe5719c3cc0a16aa94f88a5d19f8c515d4e35fae361f6f2120e".to_string(),
            0,
            "password".to_string(),
            "English".to_string(),
            NetworkType::Mainnet,
            1,
        )
        .expect("Failed to generate wallet from keys");

    // Required even though "English" was passed above.
    wallet.set_seed_language("English").unwrap();

    assert_eq!(
        wallet.get_address(0, 0).unwrap(),
        "45wsWad9EwZgF3VpxQumrUCRaEtdyyh6NG8sVD3YRVVJbK1jkpJ3zq8WHLijVzodQ22LxwkdWx7fS2a6JzaRGzkNU8K2Dhi"
    );
    assert_eq!(
        wallet.get_seed(None).unwrap(),
        "hemlock jubilee eden hacksaw boil superior inroads epoxy exhale orders cavernous second brunt saved richly lower upgrade hitched launching deepest mostly playful layout lower eden"
    );
}

#[test]
fn test_generate_view_only_from_keys() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let wallet = manager
        .generate_from_keys(
            wallet_str.to_string(),
            "45wsWad9EwZgF3VpxQumrUCRaEtdyyh6NG8sVD3YRVVJbK1jkpJ3zq8WHLijVzodQ22LxwkdWx7fS2a6JzaRGzkNU8K2Dhi".to_string(),
            "".to_string(),
            "3bc0b202cde92fe5719c3cc0a16aa94f88a5d19f8c515d4e35fae361f6f2120e".to_string(),
            0,
            "password".to_string(),
            "English".to_string(),
            NetworkType::Mainnet,
            1,
        )
        .expect("Failed to generate wallet from keys");

    wallet.set_seed_language("English").unwrap();

    assert_eq!(
        wallet.get_address(0, 0).unwrap(),
        "45wsWad9EwZgF3VpxQumrUCRaEtdyyh6NG8sVD3YRVVJbK1jkpJ3zq8WHLijVzodQ22LxwkdWx7fS2a6JzaRGzkNU8K2Dhi"
    );
    assert!(!wallet.is_deterministic().unwrap());
}

#[test]
fn test_get_seed() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let wallet = manager
        .create_wallet(wallet_str, "password", "English", NetworkType::Mainnet)
        .expect("Failed to create wallet");

    let seed = wallet.get_seed(None).expect("Failed to get seed");
    assert!(!seed.is_empty());

    let seed_with_offset = wallet.get_seed(Some("example_offset")).expect("Failed to get seed with offset");
    assert!(!seed_with_offset.is_empty());
    assert_ne!(seed, seed_with_offset);
}

#[test]
fn test_get_address() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let wallet = manager
        .create_wallet(wallet_str, "password", "English", NetworkType::Mainnet)
        .expect("Failed to create wallet");

    let address = wallet.get_address(0, 0).expect("Failed to get address");
    assert!(!address.is_empty());
}

#[test]
fn test_is_deterministic() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let wallet = manager
        .create_wallet(wallet_str, "password", "English", NetworkType::Mainnet)
        .expect("Failed to create wallet");

    assert!(wallet.is_deterministic().unwrap());
}

#[test]
fn test_wallet_creation_with_different_networks() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallets = vec![
        ("mainnet_wallet", NetworkType::Mainnet),
        ("testnet_wallet", NetworkType::Testnet),
        ("stagenet_wallet", NetworkType::Stagenet),
    ];

    for (name, net_type) in wallets {
        let wallet_path = temp_dir.path().join(name);
        let wallet_str = wallet_path.to_str().unwrap();

        let wallet = manager.create_wallet(wallet_str, "password", "English", net_type);
        assert!(wallet.is_ok(), "Failed to create wallet: {}", name);
    }
}

#[test]
fn test_multiple_address_generation() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let wallet = manager
        .create_wallet(wallet_str, "password", "English", NetworkType::Mainnet)
        .expect("Failed to create wallet");

    for i in 0..5 {
        let result = wallet.get_address(0, i);
        assert!(result.is_ok(), "Failed to get address {}: {:?}", i, result.err());
        assert!(!result.unwrap().is_empty(), "Address {} is empty", i);
    }
}

#[test]
fn test_wallet_error_variants() {
    let error = WalletError::FfiError("Test error".to_string());
    assert_eq!(format!("{}", error), "FFI error: Test error");

    let error = WalletError::NullPointer;
    assert_eq!(format!("{}", error), "null pointer from FFI");

    let error = WalletError::WalletErrorCode(2, "Sample wallet error".to_string());
    assert_eq!(format!("{}", error), "wallet error (status 2): Sample wallet error");
}

#[test]
fn test_wallet_status() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let wallet = manager
        .create_wallet(wallet_str, "password", "English", NetworkType::Mainnet)
        .expect("Failed to create wallet");

    let status = manager.get_status(wallet.ptr.as_ptr());
    assert!(status.is_ok(), "Expected status OK, got: {:?}", status.err());
}

#[test]
fn test_open_wallet() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let wallet = manager
        .create_wallet(wallet_str, "password", "English", NetworkType::Mainnet)
        .expect("Failed to create wallet");
    drop(wallet);

    let opened = manager.open_wallet(wallet_str, "password", NetworkType::Mainnet);
    assert!(opened.is_ok(), "Failed to open wallet: {:?}", opened.err());
}

#[test]
fn test_open_wallet_invalid_password() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let wallet = manager
        .create_wallet(wallet_str, "correct_password", "English", NetworkType::Mainnet)
        .expect("Failed to create wallet");
    drop(wallet);

    let result = manager.open_wallet(wallet_str, "wrong_password", NetworkType::Mainnet);
    assert!(result.is_err(), "Expected error with wrong password");
}

#[test]
fn test_open_wallet_invalid_path() {
    let (manager, _temp_dir) = setup().expect("Failed to set up test environment");

    let result = manager.open_wallet("/invalid/path/to/wallet", "password", NetworkType::Mainnet);
    match result {
        Err(WalletError::WalletErrorCode(status, msg)) => {
            assert_ne!(status, WalletStatus_Ok);
            assert!(
                msg.contains("file not found") || msg.contains("openWallet"),
                "Unexpected error: {}",
                msg
            );
        }
        Err(e) => panic!("Expected WalletErrorCode, got {:?}", e),
        Ok(_) => panic!("Expected error for non-existent wallet"),
    }
}

#[test]
fn test_get_balance() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let wallet = manager
        .create_wallet(wallet_str, "password", "English", NetworkType::Mainnet)
        .expect("Failed to create wallet");

    let balance = wallet.get_balance(0).expect("Failed to get balance");
    assert_eq!(balance.balance, 0);
    assert_eq!(balance.unlocked_balance, 0);
}

#[test]
fn test_create_account() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let wallet = manager
        .create_wallet(wallet_str, "password", "English", NetworkType::Mainnet)
        .expect("Failed to create wallet");

    wallet.create_account("Test Account").expect("Failed to create account");
}

#[test]
fn test_get_accounts() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let wallet = manager
        .create_wallet(wallet_str, "password", "English", NetworkType::Mainnet)
        .expect("Failed to create wallet");

    wallet.create_account("Account 1").unwrap();
    wallet.create_account("Account 2").unwrap();

    let accounts = wallet.get_accounts().expect("Failed to get accounts");
    assert_eq!(accounts.accounts.len(), 3);
    assert_eq!(accounts.accounts[0].label, "Primary account");
    assert_eq!(accounts.accounts[1].label, "Account 1");
    assert_eq!(accounts.accounts[2].label, "Account 2");
}

#[test]
fn test_close_wallet() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let mut wallet = manager
        .create_wallet(wallet_str, "password", "English", NetworkType::Mainnet)
        .expect("Failed to create wallet");

    wallet.close_wallet().expect("Failed to close wallet");
    wallet.close_wallet().expect("Second close should succeed");
}

#[test]
fn test_get_height() {
    let (manager, _temp_dir) = setup().expect("Failed to set up test environment");

    let height = manager.get_height().expect("Failed to get height");
    assert_eq!(height, 0);
}

#[test]
fn test_init_and_refresh() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let wallet = manager
        .create_wallet(wallet_str, "password", "English", NetworkType::Mainnet)
        .expect("Failed to create wallet");

    let config = WalletConfig {
        daemon_address: "http://localhost:18081".to_string(),
        upper_transaction_size_limit: 10000,
        daemon_username: "".to_string(),
        daemon_password: "".to_string(),
        use_ssl: false,
        light_wallet: false,
        proxy_address: "".to_string(),
    };

    wallet.init(config).expect("Failed to init wallet");
    wallet.refresh().expect("Failed to refresh wallet");
}

#[test]
fn test_set_seed_language() {
    let (manager, temp_dir) = setup().expect("Failed to set up test environment");

    let wallet_path = temp_dir.path().join("test_wallet");
    let wallet_str = wallet_path.to_str().unwrap();

    let wallet = manager
        .create_wallet(wallet_str, "password", "English", NetworkType::Mainnet)
        .expect("Failed to create wallet");

    wallet.set_seed_language("French").expect("Failed to set seed language");
}
