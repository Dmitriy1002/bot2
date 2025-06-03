use std::str::FromStr;
use std::sync::Arc;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use tokio;

use meteora_sniper_bot::swap::execute_swap;

#[tokio::test]
async fn test_execute_swap_simulation() {
    let rpc = Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com".to_string()));

    let payer = Keypair::new();
    let dummy_pubkey = Pubkey::from_str("11111111111111111111111111111111").unwrap();

    let result = execute_swap(
        rpc,
        &payer,
        dummy_pubkey, // pool
        dummy_pubkey, // user_source
        dummy_pubkey, // user_destination
        dummy_pubkey, // pool_source
        dummy_pubkey, // pool_destination
        dummy_pubkey, // pool_authority
        dummy_pubkey, // token_program
        1_000_000,
        1,
        10_000,
    )
    .await;

    assert!(
        result.is_err(),
        "Ожидалась ошибка при попытке swap с фиктивными данными"
    );
}
