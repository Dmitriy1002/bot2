use anyhow::Result;
use reqwest::Client;
use serde_json::json;

use crate::config::{NEXTBLOCK_URL, NEXTBLOCK_API_KEY};

/// Отправляет сериализованную транзакцию в NextBlock релейер
///
/// # Аргументы:
/// - `tx_base64`: транзакция в base64
/// - `tip`: для приоритизации в блоке
///
/// # Возвращает:
/// - `Ok(())` при успешной отправке
/// - `Err` при ошибке сети или некорректном ответе
pub async fn send_to_nextblock(tx_base64: &str, tip: u64) -> Result<()> {
    let client = Client::new();

    let body = json!({
        "tx": tx_base64,
        "meta": {
            "tip": tip.to_string()
        }
    });

    let res = client
        .post(NEXTBLOCK_URL)
        .header("Authorization", format!("Bearer {}", NEXTBLOCK_API_KEY))
        .json(&body)
        .send()
        .await?;

    println!("NextBlock response: {:?}", res.text().await?);

    Ok(())
}
