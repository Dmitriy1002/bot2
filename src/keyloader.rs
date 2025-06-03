use solana_sdk::signature::Keypair;
use anyhow::{Result, Context};
use bs58;

/// Декодирует приватный ключ в формате base58 в объект `Keypair`.
///
/// Используется для загрузки ключа из строки, скопированной из CLI или `.json`-файла в base58 формате.
///
/// # Аргументы:
/// - `key_str`: строка в формате base58, содержащая сериализованный приватный ключ (обычно 64 байта)
///
/// # Возвращает:
/// - `Ok(Keypair)` — при успешной декодировке и создании пары ключей
/// - `Err` — если строка невалидная или формат несовместим с `Keypair::from_bytes`
///
pub fn read_keypair_from_base58_string(key_str: &str) -> Result<Keypair> {
    let data = bs58::decode(key_str)
        .into_vec()
        .context("Невозможно декодировать base58 строку")?;

    let keypair = Keypair::from_bytes(&data).context("Невалидный формат ключа")?;
    Ok(keypair)
}
