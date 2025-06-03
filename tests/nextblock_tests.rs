use meteora_sniper_bot::nextblock::send_to_nextblock;

#[tokio::test]
async fn test_send_to_nextblock_invalid_tx() {
    let tx_base64 = "invalid_base64_string";
    let tip = 42;

    let result = send_to_nextblock(tx_base64, tip).await;

    assert!(
        result.is_err(),
        "NextBlock должен вернуть ошибку при невалидной транзакции"
    );
}