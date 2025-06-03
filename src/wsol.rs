use std::str::FromStr;

use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::program_pack::Pack;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token::{
    instruction::{initialize_account, sync_native},
    state::Account,
};

use crate::config::WSOL_MINT;

pub async fn ensure_wsol_account(
    rpc: &RpcClient,
    payer: &Keypair,
) -> Result<Pubkey> {
    let wsol_mint = Pubkey::from_str(WSOL_MINT)?;

    let token_account = Keypair::new();
    let rent_exemption = rpc
        .get_minimum_balance_for_rent_exemption(Account::LEN)
        .await?;

    let create_acc_ix = system_instruction::create_account(
        &payer.pubkey(),
        &token_account.pubkey(),
        rent_exemption + LAMPORTS_PER_SOL / 10,
        Account::LEN as u64,
        &spl_token::id(),
    );

    let init_acc_ix = initialize_account(
        &spl_token::id(),
        &token_account.pubkey(),
        &wsol_mint,
        &payer.pubkey(),
    )?;

    let sync_ix = sync_native(&spl_token::id(), &token_account.pubkey())?;

    let recent_blockhash = rpc.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &[create_acc_ix, init_acc_ix, sync_ix],
        Some(&payer.pubkey()),
        &[payer, &token_account],
        recent_blockhash,
    );

    rpc.send_and_confirm_transaction(&tx).await?;

    println!("WSOL аккаунт создан: {}", token_account.pubkey());
    Ok(token_account.pubkey())
}