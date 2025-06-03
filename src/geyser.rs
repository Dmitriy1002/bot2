use std::{
    collections::HashSet,
    str::FromStr,
    sync::{Arc, RwLock},
};

use anyhow::Result;
use futures_util::sink::SinkExt;
use prost::Message;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    message::VersionedMessage,
    pubkey::Pubkey,
    signature::Keypair,
    transaction::{VersionedTransaction},
};
use tokio_stream::StreamExt;
use yellowstone_grpc_client::{ClientTlsConfig, GeyserGrpcClient};
use yellowstone_grpc_proto::prelude::{
    subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest, SubscribeUpdate,
};

use crate::config::{GRPC_URL, RELAYER_TIP_LAMPORTS, WSOL_MINT};
use crate::swap::execute_swap;

fn is_initialize_instruction(data: &[u8]) -> bool {
    !data.is_empty() && data[0] == 2
}

/// Подписывается на поток транзакций из Geyser GRPC и отслеживает
/// появление новых пулов с участием WSOL, чтобы автоматически выполнить swap
///
/// # Аргументы
/// * `rpc_client` — клиент RPC для взаимодействия с Solana
/// * `payer` — ключ, с которого отправляется транзакция swap
///
/// # Возвращает
/// `Result<()>` — успешное выполнение или ошибка при подписке / swap
pub async fn monitor_liquidity_additions(
    rpc_client: Arc<RpcClient>,
    payer: Arc<Keypair>,
) -> Result<()> {
    let wsol_mint = Pubkey::from_str(WSOL_MINT)?;

    let grpc_builder = GeyserGrpcClient::build_from_static(GRPC_URL)
        .tls_config(ClientTlsConfig::new().with_native_roots())?;

    // Подключение к GRPC серверу и подписка
    let mut client = grpc_builder.connect().await?;
    let (mut sender, mut stream) = client.subscribe().await?;

    sender
        .send(SubscribeRequest {
            commitment: Some(CommitmentLevel::Processed as i32),
            ..Default::default()
        })
        .await?;

    println!("Ожидание транзакций от Meteora Pools...");

    let seen_pools: Arc<RwLock<HashSet<Pubkey>>> = Arc::new(RwLock::new(HashSet::new()));

    // Основной цикл обработки входящих транзакций
    while let Some(resp) = stream.next().await {
        match resp {
            Ok(SubscribeUpdate {
                update_oneof: Some(UpdateOneof::Transaction(tx_info)),
                ..
            }) => {
                if let Some(tx) = &tx_info.transaction {
                    let mut buf = Vec::new();
                    tx.encode(&mut buf)?;

                    if let Ok(versioned_tx) =
                        bincode::deserialize::<VersionedTransaction>(&buf)
                    {
                        if let VersionedMessage::V0(message) = &versioned_tx.message {
                            for instr in &message.instructions {
                                if is_initialize_instruction(&instr.data) {
                                    let keys = &message.account_keys;
                                    let accs = &instr.accounts;

                                    if accs.len() < 10 {
                                        continue;
                                    }

                                    // Определение, есть ли WSOL в паре токенов
                                    let token_a = keys[accs[8] as usize];
                                    let token_b = keys[accs[9] as usize];

                                    let (target_mint, is_valid_pair) = if token_a == wsol_mint {
                                        (token_b, true)
                                    } else if token_b == wsol_mint {
                                        (token_a, true)
                                    } else {
                                        (Pubkey::default(), false)
                                    };

                                    if !is_valid_pair {
                                        continue;
                                    }

                                    // Проверка, был ли пул уже обработан
                                    let pool = keys[accs[2] as usize];
                                    let mut seen = seen_pools.write().unwrap();
                                    if seen.contains(&pool) {
                                        continue;
                                    }
                                    seen.insert(pool);

                                    println!("Новый пул с WSOL: {}", pool);
                                    println!("Токен к покупке: {}", target_mint);

                                    let user_source = keys[accs[0] as usize];
                                    let user_dest = keys[accs[1] as usize];
                                    let pool_source = keys[accs[2] as usize];
                                    let pool_dest = keys[accs[3] as usize];
                                    let pool_auth = keys[accs[4] as usize];
                                    let token_prog = keys[accs[5] as usize];

                                    // Выполнение свапа
                                    let _ = execute_swap(
                                        rpc_client.clone(),
                                        &payer,
                                        pool,
                                        user_source,
                                        user_dest,
                                        pool_source,
                                        pool_dest,
                                        pool_auth,
                                        token_prog,
                                        1_000_000,
                                        1,
                                        RELAYER_TIP_LAMPORTS,
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("Ошибка в потоке транзакций: {:?}", e);
            }
            _ => {}
        }
    }

    Ok(())
}
