# ⚡ QuantumFlow - High-Frequency Trading Engine

[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)

[English](#english) | [Português](#português)

---

## English

### 🚀 Overview

**QuantumFlow** is a high-frequency trading (HFT) engine built in Rust, designed for low-latency order execution and real-time market data processing. It includes a matching engine, risk management system, and exchange connectivity via WebSocket.

### ✨ Key Features

- **Low-Latency Matching**: FIFO price-time priority order matching with partial fill support
- **Binance Connectivity**: WebSocket connector for real-time market data streaming
- **Risk Management**: Position limits, circuit breakers, and P&L tracking
- **Backtesting**: Historical data simulation with performance metrics (Sharpe, drawdown, win rate)
- **Async Runtime**: Built on Tokio for concurrent order processing
- **Decimal Precision**: Uses `rust_decimal` for accurate financial arithmetic

### 🏗️ Architecture

![Architecture Diagram](docs/images/architecture.png)

The system is organized into modular layers:

1. **Connectors Layer**: WebSocket connection to Binance
2. **Core Engine**: Order book management and matching logic
3. **Risk Management**: Position tracking and limit enforcement
4. **Analytics**: Backtesting and performance analysis

### 📊 Matching Flow

![Matching Flow](docs/images/matching_flow.png)

### 🛠️ Installation

#### Prerequisites

- Rust 1.90+ ([Install Rust](https://www.rust-lang.org/tools/install))
- Build essentials (gcc, pkg-config, libssl-dev)

#### Build from Source

```bash
git clone https://github.com/gabriellafis/quantumflow.git
cd quantumflow
cargo build --release
```

### 🎯 Quick Start

#### 1. Run Demo Trading Simulation

```bash
cargo run --release -- demo
```

Output:
```
INFO quantumflow: Running demo trading simulation
INFO quantumflow::engine::matching: Trade executed: BTCUSD @ 50200 qty 1
INFO quantumflow::risk::manager: Position updated: BTCUSD SELL @ 50200 qty 1, PnL change: 0
...
Risk Metrics:
  Daily PnL: 0
  Total Exposure: 0
  Circuit Breaker: false
```

#### 2. Start Matching Engine

```bash
cargo run --release -- match --symbol BTCUSD
```

#### 3. Stream Live Market Data

```bash
# Stream ticker data from Binance
cargo run --release -- stream --symbol btcusdt --stream-type ticker

# Stream order book data
cargo run --release -- stream --symbol btcusdt --stream-type orderbook
```

#### 4. Run Backtest

```bash
cargo run --release -- backtest --file data/sample/btc_historical.csv
```

### 📚 Usage Examples

#### Programmatic Trading

```rust
use quantumflow::{MatchingEngine, Order, OrderType, Side};
use rust_decimal::Decimal;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (trade_tx, mut trade_rx) = mpsc::unbounded_channel();
    let engine = MatchingEngine::new(trade_tx);

    // Create buy order
    let buy_order = Order::new(
        "BTCUSD".to_string(),
        Side::Buy,
        OrderType::Limit,
        Decimal::from(50000),
        Decimal::from(1),
    );

    // Submit order
    let result = engine.submit_order(buy_order).await?;
    println!("Order status: {:?}", result.status);

    Ok(())
}
```

#### Risk Management

```rust
use quantumflow::risk::manager::{RiskManager, RiskLimits};
use rust_decimal::Decimal;

let limits = RiskLimits {
    max_order_size: Decimal::from(10),
    max_position_size: Decimal::from(100),
    max_daily_loss: Decimal::from(10000),
    max_leverage: Decimal::from(5),
};

let risk_manager = RiskManager::new(limits);

// Check order against risk limits
if let Err(e) = risk_manager.check_order(&order) {
    println!("Order rejected: {}", e);
}
```

### ⚙️ Configuration

Risk limits are configured programmatically via `RiskLimits`:

```rust
let limits = RiskLimits {
    max_order_size: Decimal::from(10),
    max_position_size: Decimal::from(100),
    max_daily_loss: Decimal::from(10000),
    max_leverage: Decimal::from(5),
};
```

### 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_orderbook_add_and_match
```

### 📈 Benchmarks

Run benchmarks on your own hardware:

```bash
cargo bench
```

Benchmarks cover order submission, order matching, and order book snapshots using the `criterion` framework.

### 🗂️ Project Structure

```
quantumflow/
├── src/
│   ├── engine/
│   │   ├── orderbook.rs      # Order book implementation
│   │   └── matching.rs       # Matching engine logic
│   ├── connectors/
│   │   └── binance.rs        # Binance WebSocket connector
│   ├── risk/
│   │   └── manager.rs        # Risk management system
│   ├── backtest/
│   │   └── engine.rs         # Backtesting framework
│   ├── utils/
│   │   └── types.rs          # Core data types
│   ├── lib.rs                # Library exports
│   └── main.rs               # CLI application
├── tests/
│   └── integration_test.rs   # Integration tests
├── benches/
│   ├── orderbook_bench.rs    # Order book benchmarks
│   └── matching_engine_bench.rs
├── examples/
│   └── simple_trading.rs     # Usage examples
├── docs/
│   ├── architecture.mmd      # Architecture diagram (Mermaid)
│   ├── matching_flow.mmd     # Matching flow diagram (Mermaid)
│   └── images/               # Rendered diagrams
├── Cargo.toml
├── LICENSE
└── README.md
```

### 🤝 Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

### 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### 👤 Author

**Gabriel Demetrios Lafis**
- Systems Analyst & Developer
- IT Manager
- Cybersecurity Specialist
- Business Intelligence / Business Analyst
- Data Analyst & Data Scientist

---

## Português

### 🚀 Visão Geral

**QuantumFlow** é um motor de negociação de alta frequência (HFT) construído em Rust, projetado para execução de ordens com baixa latência e processamento de dados de mercado em tempo real. Possui um motor de correspondência, sistema de gestão de risco e conectividade com exchange via WebSocket.

### ✨ Principais Recursos

- **Correspondência de Baixa Latência**: Prioridade FIFO preço-tempo com suporte a preenchimento parcial
- **Conectividade com Binance**: Conector WebSocket para streaming de dados de mercado em tempo real
- **Gestão de Risco**: Limites de posição, circuit breakers e rastreamento de P&L
- **Backtesting**: Simulação com dados históricos e métricas de desempenho (Sharpe, drawdown, win rate)
- **Runtime Assíncrono**: Construído sobre Tokio para processamento concorrente de ordens
- **Precisão Decimal**: Usa `rust_decimal` para aritmética financeira precisa

### 🏗️ Arquitetura

![Diagrama de Arquitetura](docs/images/architecture.png)

O sistema é organizado em camadas modulares:

1. **Camada de Conectores**: Conexão WebSocket com a Binance
2. **Motor Principal**: Gerenciamento de livro de ordens e lógica de correspondência
3. **Gestão de Risco**: Rastreamento de posições e aplicação de limites
4. **Analytics**: Backtesting e análise de desempenho

### 📊 Fluxo de Correspondência

![Fluxo de Correspondência](docs/images/matching_flow.png)

### 🛠️ Instalação

#### Pré-requisitos

- Rust 1.90+ ([Instalar Rust](https://www.rust-lang.org/tools/install))
- Build essentials (gcc, pkg-config, libssl-dev)

#### Compilar do Código Fonte

```bash
git clone https://github.com/gabriellafis/quantumflow.git
cd quantumflow
cargo build --release
```

### 🎯 Início Rápido

#### 1. Executar Simulação de Trading Demo

```bash
cargo run --release -- demo
```

#### 2. Iniciar Motor de Correspondência

```bash
cargo run --release -- match --symbol BTCUSD
```

#### 3. Transmitir Dados de Mercado ao Vivo

```bash
# Transmitir dados de ticker da Binance
cargo run --release -- stream --symbol btcusdt --stream-type ticker

# Transmitir dados de livro de ordens
cargo run --release -- stream --symbol btcusdt --stream-type orderbook
```

#### 4. Executar Backtest

```bash
cargo run --release -- backtest --file data/sample/btc_historical.csv
```

### 🧪 Testes

```bash
# Executar todos os testes
cargo test

# Executar com saída
cargo test -- --nocapture
```

### 📈 Benchmarks

Execute os benchmarks no seu próprio hardware:

```bash
cargo bench
```

Os benchmarks cobrem submissão de ordens, correspondência de ordens e snapshots do livro de ordens usando o framework `criterion`.

### 📄 Licença

Este projeto está licenciado sob a Licença MIT - consulte o arquivo [LICENSE](LICENSE) para detalhes.

### 👤 Autor

**Gabriel Demetrios Lafis**
- Analista e Desenvolvedor de Sistemas
- Gestor de Tecnologia da Informação
- Especialista em Segurança Cibernética
- Business Intelligence / Business Analyst
- Analista e Cientista de Dados
