use solana_sdk::signature::Keypair;
use meteora_sniper_bot::wsol::ensure_wsol_account;
use solana_client::nonblocking::rpc_client::RpcClient;

#[tokio::test]
async fn test_ensure_wsol_account_invalid_rpc() {
    let rpc_client = RpcClient::new("https://invalid-rpc.test".to_string());
    let dummy_payer = Keypair::new();

    let result = ensure_wsol_account(&rpc_client, &dummy_payer).await;

    assert!(result.is_err(), "Должна быть ошибка при невалидном RPC");
}
