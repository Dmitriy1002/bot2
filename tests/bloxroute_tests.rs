use meteora_sniper_bot::bloxroute::send_to_bloxroute;

#[tokio::test]
async fn test_send_to_bloxroute_invalid_tx() {
    let tx_base64 = "invalid_base64";
    let tip = 100_000;

    let result = send_to_bloxroute(tx_base64, tip).await;

    assert!(
        result.is_err(),
        "Bloxroute должен вернуть ошибку при невалидной транзакции"
    );
}