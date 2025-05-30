use solana_sdk::{signature::Keypair, signer::Signer};
use anyhow::{Result, Context};
use bs58;

pub fn read_keypair_from_base58_string(key_str: &str) -> Result<Keypair> {
    let data = bs58::decode(key_str)
        .into_vec()
        .context("Невозможно декодировать base58 строку")?;

    let keypair = Keypair::from_bytes(&data).context("Невалидный формат ключа")?;
    Ok(keypair)
}