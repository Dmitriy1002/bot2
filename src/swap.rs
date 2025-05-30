use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use bincode::serialize;
use reqwest::Client;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::{Transaction, VersionedTransaction},
};
use tokio::join;

use crate::config::{
    BLOXROUTE_API_KEY, BLOXROUTE_URL, METEORA_PROGRAM_ID, NEXTBLOCK_API_KEY, NEXTBLOCK_URL,
};

#[derive(Debug)]
struct SwapInstructionData {
    amount_in: u64,
    minimum_amount_out: u64,
}

impl SwapInstructionData {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(4);
        buf.extend(&self.amount_in.to_le_bytes());
        buf.extend(&self.minimum_amount_out.to_le_bytes());
        buf
    }
}

async fn send_tx_to_relayer(
    url: &str,
    api_key: &str,
    tx: &VersionedTransaction,
) -> anyhow::Result<()> {
    let client = Client::new();
    let tx_bytes = serialize(tx)?;
    let tx_base64 = general_purpose::STANDARD.encode(&tx_bytes);

    let body = serde_json::json!({ "transaction": tx_base64 });

    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", api_key)
        .json(&body)
        .send()
        .await?;

    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        anyhow::bail!("Relayer {} error: {}", url, text);
    }

    Ok(())
}

pub async fn execute_swap(
    rpc: Arc<RpcClient>,
    payer: &Keypair,
    pool: Pubkey,
    user_source: Pubkey,
    user_destination: Pubkey,
    pool_source: Pubkey,
    pool_destination: Pubkey,
    pool_authority: Pubkey,
    token_program: Pubkey,
    amount_in: u64,
    min_out: u64,
) -> Result<()> {
    println!("Составляем swap через Meteora");

    let ix_data = SwapInstructionData {
        amount_in,
        minimum_amount_out: min_out,
    }
    .serialize();

    let ix = Instruction {
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

    let blockhash = rpc.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[payer], blockhash);
    let versioned_tx = VersionedTransaction::from(tx);

    let (rpc_res, blox_res, next_res) = join!(
        rpc.send_and_confirm_transaction(&versioned_tx),
        send_tx_to_relayer(BLOXROUTE_URL, BLOXROUTE_API_KEY, &versioned_tx),
        send_tx_to_relayer(NEXTBLOCK_URL, NEXTBLOCK_API_KEY, &versioned_tx),
    );

    match rpc_res {
        Ok(sig) => println!("Покупка отправлена через Solana RPC: {}", sig),
        Err(e) => eprintln!("Ошибка Solana RPC: {:?}", e),
    }

    if let Err(e) = blox_res {
        eprintln!("Ошибка отправки в Bloxroute: {:?}", e);
    }

    if let Err(e) = next_res {
        eprintln!("Ошибка отправки в NextBlock: {:?}", e);
    }

    Ok(())
}