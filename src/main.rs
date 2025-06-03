mod config;

use config::{RPC_URL, PRIVATE_KEY_BASE58};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;

mod geyser;
mod swap;
mod keyloader;
mod wsol;
mod bloxroute;
mod nextblock;

#[tokio::main]
async fn main() {
    // Инициализация RPC клиента с уровнем подтверждения "confirmed"
    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        RPC_URL.to_string(),
        CommitmentConfig::confirmed(),
    ));

    // Загрузка приватного ключа
    let payer = match keyloader::read_keypair_from_base58_string(PRIVATE_KEY_BASE58) {
        Ok(k) => Arc::new(k),
        Err(e) => {
            eprintln!("Ошибка загрузки ключа: {}", e);
            return;
        }
    };

    // Создание WSOL аккаунта при необходимости
    if let Err(e) = wsol::ensure_wsol_account(&rpc_client, &payer).await {
        eprintln!("Ошибка создания WSOL аккаунта: {:?}", e);
        return;
    }

    println!("Запуск отслеживания ликвидности через Meteora...");

    // Запуск мониторинга транзакций через Geyser
    if let Err(e) = geyser::monitor_liquidity_additions(rpc_client.clone(), payer.clone()).await {
        eprintln!("Ошибка мониторинга: {:?}", e);
    }
}