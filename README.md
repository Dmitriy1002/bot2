# 🔥 Solana Meteora Sniper Bot

Бот, отслеживающий новые пулы в [Meteora Pools](https://meteora.ag) и совершающий мгновенную покупку токена в пуле, где участвует WSOL. 
Транзакции отправляются через Solana RPC и два MEV-ретранслятора: **Bloxroute** и **NextBlock**.

---

## 📌 Основная логика

### 1. 🔐 Загрузка ключа

```rust
let payer = keyloader::read_keypair_from_base58_string(PRIVATE_KEY_BASE58)?;
```

###  2. 🪙 Создание временного WSOL-аккаунта

```rust
Файл: wsol.rs

Создаётся временный spl_token::Account с WSOL.
Счёт пополняется на 0.1 SOL.
Инициализируется как WSOL-аккаунт через initialize_account.
Синхронизируется через sync_native.
```

### 3. 🛰 Подписка на Geyser gRPC
```rust
Файл: geyser.rs

let grpc_builder = GeyserGrpcClient::build_from_static(GRPC_URL);
let (mut sender, mut stream) = client.subscribe().await?;
Geyser-клиент подключается к grpc.ny.shyft.to.
Подписка на CommitmentLevel::Processed.
Получение всех транзакций в реальном времени.
```

### 4. 🔍 Поиск пула с WSOL
```rust
if is_initialize_instruction(&instr.data) { ... }

Определение initialize_pool по сигнатуре.
Проверка, участвует ли WSOL в паре.
Исключение повторной покупки пула через HashSet.
```

### 5. ⚙️ Сбор параметров swap
```rust
Из транзакции извлекаются:
user_source, user_destination
pool_source, pool_destination
pool_authority, token_program
```

### 6. 🔁 Выполнение swap
```rust
Файл: swap.rs

execute_swap(...).await;
Сбор инструкции swap (код операции = 4).

Подпись транзакции.

Отправка одновременно в:
Solana RPC
Bloxroute
NextBlock
```

### 7. 📡 Отправка в ретрансляторы
```rust
Файлы:
bloxroute.rs
nextblock.rs

Bloxroute:
Метод blxr_tx, канал solana-mainnet, поля tip, mev, max_block_delay.

NextBlock:
POST-запрос с tx и meta.tip
```

### Структура проекта
```rust
├── src/
│   ├── main.rs          # Точка запуска
│   ├── config.rs        # Конфигурация
│   ├── keyloader.rs     # Загрузка ключей
│   ├── geyser.rs        # Мониторинг пулов
│   ├── swap.rs          # Логика swap-инструкции
│   ├── wsol.rs          # Инициализация WSOL
│   ├── bloxroute.rs     # Отправка в Bloxroute
│   └── nextblock.rs     # Отправка в NextBlock
└── tests/
├── wsol_tests.rs
└── bloxroute_tests.rs
```

### ⚙️ Конфигурация (config.rs)
```rust
pub const RPC_URL: &str = "https://api.mainnet-beta.solana.com";
pub const PRIVATE_KEY_BASE58: &str = "...";
pub const BLOXROUTE_API_KEY: &str = "...";
pub const NEXTBLOCK_API_KEY: &str = "...";
pub const METEORA_PROGRAM_ID: &str = "...";
pub const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";
pub const GRPC_URL: &str = "https://grpc.ny.shyft.to";
```


### 🚀 Запуск
```rust
cargo run --release

Происходит:
Инициализация WSOL-аккаунта.
Подключение к Geyser.
Мониторинг всех транзакций.
Покупка токена при создании пула с WSOL.
Отправка swap через 3 канала.
```
