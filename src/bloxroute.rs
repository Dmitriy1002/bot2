use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::{json, Value};

use crate::config::{BLOXROUTE_URL, BLOXROUTE_API_KEY};

/// Отправляет base64-сериализованную транзакцию в Bloxroute MEV endpoint для Solana.
///
/// Используется для приоритетной обработки транзакции через релейер.
/// Требует, чтобы в саму транзакцию был встроен `ComputeBudgetInstruction::set_compute_unit_price(tip)`,
/// иначе `tip` не будет иметь эффекта.
///
/// # Аргументы:
/// - `tx_base64`: сериализованная транзакция в формате base64
/// - `tip`: чаевые в микролампортах (u64), передающиеся в теле запроса и внутри транзакции
///
/// # Возвращает:
/// - `Ok(())` при успешной отправке и отсутствии ошибок
/// - `Err` при сетевых ошибках или если Bloxroute вернул `error` в ответе
pub async fn send_to_bloxroute(tx_base64: &str, tip: u64) -> Result<()> {
    let client = Client::new();

    let body = json!({
        "transaction": tx_base64,
        "channel": "solana-mainnet",
        "mev": true,
        "max_block_delay": 2,
        "tip": tip.to_string()
    });

    let res = client
        .post(BLOXROUTE_URL)
        .header("Authorization", format!("Bearer {}", BLOXROUTE_API_KEY))
        .json(&body)
        .send()
        .await?;

    let text = res.text().await?;
    println!("Bloxroute response: {}", text);

    let json: Value = serde_json::from_str(&text)?;

    if json.get("error").is_some() {
        return Err(anyhow!("Bloxroute returned error: {}", json));
    }

    Ok(())
}