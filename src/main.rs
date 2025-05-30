mod config;

use config::{RPC_URL, PRIVATE_KEY_BASE58};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;

mod geyser;
mod swap;
mod keyloader;
mod wsol;

#[tokio::main]
async fn main() {
    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        RPC_URL.to_string(),
        CommitmentConfig::confirmed(),
    ));

    let payer = match keyloader::read_keypair_from_base58_string(PRIVATE_KEY_BASE58) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("Ошибка загрузки ключа: {}", e);
            return;
        }
    };

    let _ = wsol::ensure_wsol_account(&rpc_client, &payer).await;

    println!("🔍 Запуск отслеживания ликвидности через Meteora...");
    if let Err(e) = geyser::monitor_liquidity_additions(rpc_client.clone()).await {
        eprintln!("Ошибка мониторинга: {:?}", e);
    }
}