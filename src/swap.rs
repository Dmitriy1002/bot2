use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use bincode::serialize;
use solana_client::nonblocking::rpc_client::RpcClient;

use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::{Transaction, VersionedTransaction},
};
use tokio::join;

use crate::bloxroute::send_to_bloxroute;
use crate::nextblock::send_to_nextblock;
use crate::config::METEORA_PROGRAM_ID;

#[derive(Debug)]
struct SwapInstructionData {
    amount_in: u64,
    minimum_amount_out: u64,
}

impl SwapInstructionData {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(4); // opcode для swap'а в Meteora
        buf.extend(&self.amount_in.to_le_bytes());
        buf.extend(&self.minimum_amount_out.to_le_bytes());
        buf
    }
}

/// Выполняет swap через программу Meteora
///
/// Отправляет транзакцию через:
/// 1. Стандартный Solana RPC
/// 2. Bloxroute MEV endpoint
/// 3. NextBlock endpoint
///
/// # Аргументы
/// * `rpc` — RPC клиент
/// * `payer` — аккаунт, подписывающий транзакцию
/// * `_pool` — публичный ключ пула (не используется напрямую, но может быть полезен логически)
/// * `user_source` — аккаунт, с которого списываются токены
/// * `user_destination` — аккаунт, на который зачисляются токены
/// * `pool_source` — аккаунт пула (откуда берётся токен)
/// * `pool_destination` — аккаунт пула (куда кладётся токен)
/// * `pool_authority` — authority пула
/// * `token_program` — SPL Token программа
/// * `amount_in` — количество входных токенов
/// * `min_out` — минимальное количество выходных токенов
/// * `tip` — повышени приоритета
pub async fn execute_swap(
    rpc: Arc<RpcClient>,
    payer: &Keypair,
    _pool: Pubkey,
    user_source: Pubkey,
    user_destination: Pubkey,
    pool_source: Pubkey,
    pool_destination: Pubkey,
    pool_authority: Pubkey,
    token_program: Pubkey,
    amount_in: u64,
    min_out: u64,
    tip: u64,
) -> Result<()> {
    println!("Составляем swap через Meteora");

    let ix_data = SwapInstructionData {
        amount_in,
        minimum_amount_out: min_out,
    }
    .serialize();

    // Основная инструкция swap через Meteora
    let swap_ix = Instruction {
        program_id: Pubkey::from_str(METEORA_PROGRAM_ID)?,
        accounts: vec![
            AccountMeta::new(user_source, false),
            AccountMeta::new(user_destination, false),
            AccountMeta::new(pool_source, false),
            AccountMeta::new(pool_destination, false),
            AccountMeta::new_readonly(pool_authority, false),
            AccountMeta::new_readonly(token_program, false),
        ],
        data: ix_data,
    };

    let compute_budget_ix = ComputeBudgetInstruction::set_compute_unit_price(tip);

    let blockhash = rpc.get_latest_blockhash().await?;

    let tx = Transaction::new_signed_with_payer(
        &[compute_budget_ix, swap_ix],
        Some(&payer.pubkey()),
        &[payer],
        blockhash,
    );

    let versioned_tx = VersionedTransaction::from(tx);

    let tx_bytes = serialize(&versioned_tx)?;
    let tx_base64 = general_purpose::STANDARD.encode(&tx_bytes);

    let (rpc_res, blox_res, next_res) = join!(
        rpc.send_and_confirm_transaction(&versioned_tx),
        send_to_bloxroute(&tx_base64, tip),
        send_to_nextblock(&tx_base64, tip),
    );

    let mut errors = vec![];

    match rpc_res {
        Ok(sig) => println!("Покупка отправлена через Solana RPC: {}", sig),
        Err(e) => {
            eprintln!("Ошибка Solana RPC: {:?}", e);
            errors.push(anyhow!("Solana RPC error: {:?}", e));
        }
    }

    if let Err(e) = blox_res {
        eprintln!("Ошибка отправки в Bloxroute: {:?}", e);
        errors.push(anyhow!("Bloxroute error: {:?}", e));
    }

    if let Err(e) = next_res {
        eprintln!("Ошибка отправки в NextBlock: {:?}", e);
        errors.push(anyhow!("NextBlock error: {:?}", e));
    }

    if !errors.is_empty() {
        return Err(anyhow!("Ошибка при выполнении свапа: {:?}", errors));
    }

    Ok(())
}