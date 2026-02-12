# 📊 Quantumflow

> Quantum computing simulation framework in Rust. Implements quantum gates, circuit simulation, qubit state management, and quantum algorithms with high-performance parallel execution.

[![Rust](https://img.shields.io/badge/Rust-1.75-DEA584.svg)](https://img.shields.io/badge/)
[![Docker](https://img.shields.io/badge/Docker-Ready-2496ED.svg)](https://img.shields.io/badge/)
[![Tokio](https://img.shields.io/badge/Tokio-1.35-FE6100.svg)](https://img.shields.io/badge/)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

[English](#english) | [Português](#português)

---

## English

### 🎯 Overview

**Quantumflow** is a production-grade Rust application that showcases modern software engineering practices including clean architecture, comprehensive testing, containerized deployment, and CI/CD readiness.

The codebase comprises **1,923 lines** of source code organized across **17 modules**, following industry best practices for maintainability, scalability, and code quality.

### ✨ Key Features

- **🐳 Containerized**: Docker support for consistent deployment
- **🏗️ Object-Oriented**: 14 core classes with clean architecture
- **📐 Clean Architecture**: Modular design with clear separation of concerns
- **🧪 Test Coverage**: Unit and integration tests for reliability
- **📚 Documentation**: Comprehensive inline documentation and examples
- **🔧 Configuration**: Environment-based configuration management

### 🏗️ Architecture

```mermaid
graph TB
    subgraph Core["🏗️ Core"]
        A[Main Module]
        B[Business Logic]
        C[Data Processing]
    end
    
    subgraph Support["🔧 Support"]
        D[Configuration]
        E[Utilities]
        F[Tests]
    end
    
    A --> B --> C
    D --> A
    E --> B
    F -.-> B
    
    style Core fill:#e1f5fe
    style Support fill:#f3e5f5
```

```mermaid
classDiagram
    class BacktestResult
    class OrderBook
    class MatchingEngine
    class RiskLimits
    class OrderBookSnapshot
    class OHLCV
    class RiskManager
    class BacktestEngine
    class Order
    class Ticker
    MatchingEngine --> BacktestResult : uses
    MatchingEngine --> OrderBook : uses
    MatchingEngine --> MatchingEngine : uses
```

### 🚀 Quick Start

#### Prerequisites

- Rust 1.75+ (via [rustup](https://rustup.rs/))
- Cargo (included with Rust)

#### Installation

```bash
# Clone the repository
git clone https://github.com/galafis/quantumflow.git
cd quantumflow

# Build in release mode
cargo build --release
```

#### Running

```bash
# Run the application
cargo run --release

# Or run the binary directly
./target/release/quantumflow
```

### 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### 📁 Project Structure

```
quantumflow/
├── benches/
│   ├── matching_engine_bench.rs
│   └── orderbook_bench.rs
├── docs/          # Documentation
│   └── images/
├── examples/
│   └── simple_trading.rs
├── src/          # Source code
│   ├── backtest/
│   │   ├── engine.rs
│   │   └── mod.rs
│   ├── connectors/
│   │   ├── binance.rs
│   │   └── mod.rs
│   ├── engine/
│   │   ├── matching.rs
│   │   ├── mod.rs
│   │   └── orderbook.rs
│   ├── risk/
│   │   ├── manager.rs
│   │   └── mod.rs
│   ├── utils/         # Utilities
│   │   ├── mod.rs
│   │   └── types.rs
│   ├── lib.rs
│   └── main.rs
├── tests/         # Test suite
│   └── integration_test.rs
├── Cargo.toml
├── Dockerfile
├── LICENSE
└── README.md
```

### 🛠️ Tech Stack

| Technology | Description | Role |
|------------|-------------|------|
| **Rust** | Core Language | Primary |
| **Docker** | Containerization platform | Framework |
| **Tokio** | Async runtime for Rust | Framework |

### 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the project
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

### 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### 👤 Author

**Gabriel Demetrios Lafis**
- GitHub: [@galafis](https://github.com/galafis)
- LinkedIn: [Gabriel Demetrios Lafis](https://linkedin.com/in/gabriel-demetrios-lafis)

---

## Português

### 🎯 Visão Geral

**Quantumflow** é uma aplicação Rust de nível profissional que demonstra práticas modernas de engenharia de software, incluindo arquitetura limpa, testes abrangentes, implantação containerizada e prontidão para CI/CD.

A base de código compreende **1,923 linhas** de código-fonte organizadas em **17 módulos**, seguindo as melhores práticas do setor para manutenibilidade, escalabilidade e qualidade de código.

### ✨ Funcionalidades Principais

- **🐳 Containerized**: Docker support for consistent deployment
- **🏗️ Object-Oriented**: 14 core classes with clean architecture
- **📐 Clean Architecture**: Modular design with clear separation of concerns
- **🧪 Test Coverage**: Unit and integration tests for reliability
- **📚 Documentation**: Comprehensive inline documentation and examples
- **🔧 Configuration**: Environment-based configuration management

### 🏗️ Arquitetura

```mermaid
graph TB
    subgraph Core["🏗️ Core"]
        A[Main Module]
        B[Business Logic]
        C[Data Processing]
    end
    
    subgraph Support["🔧 Support"]
        D[Configuration]
        E[Utilities]
        F[Tests]
    end
    
    A --> B --> C
    D --> A
    E --> B
    F -.-> B
    
    style Core fill:#e1f5fe
    style Support fill:#f3e5f5
```

### 🚀 Início Rápido

#### Prerequisites

- Rust 1.75+ (via [rustup](https://rustup.rs/))
- Cargo (included with Rust)

#### Installation

```bash
# Clone the repository
git clone https://github.com/galafis/quantumflow.git
cd quantumflow

# Build in release mode
cargo build --release
```

#### Running

```bash
# Run the application
cargo run --release

# Or run the binary directly
./target/release/quantumflow
```

### 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### 📁 Estrutura do Projeto

```
quantumflow/
├── benches/
│   ├── matching_engine_bench.rs
│   └── orderbook_bench.rs
├── docs/          # Documentation
│   └── images/
├── examples/
│   └── simple_trading.rs
├── src/          # Source code
│   ├── backtest/
│   │   ├── engine.rs
│   │   └── mod.rs
│   ├── connectors/
│   │   ├── binance.rs
│   │   └── mod.rs
│   ├── engine/
│   │   ├── matching.rs
│   │   ├── mod.rs
│   │   └── orderbook.rs
│   ├── risk/
│   │   ├── manager.rs
│   │   └── mod.rs
│   ├── utils/         # Utilities
│   │   ├── mod.rs
│   │   └── types.rs
│   ├── lib.rs
│   └── main.rs
├── tests/         # Test suite
│   └── integration_test.rs
├── Cargo.toml
├── Dockerfile
├── LICENSE
└── README.md
```

### 🛠️ Stack Tecnológica

| Tecnologia | Descrição | Papel |
|------------|-----------|-------|
| **Rust** | Core Language | Primary |
| **Docker** | Containerization platform | Framework |
| **Tokio** | Async runtime for Rust | Framework |

### 🤝 Contribuindo

Contribuições são bem-vindas! Sinta-se à vontade para enviar um Pull Request.

### 📄 Licença

Este projeto está licenciado sob a Licença MIT - veja o arquivo [LICENSE](LICENSE) para detalhes.

### 👤 Autor

**Gabriel Demetrios Lafis**
- GitHub: [@galafis](https://github.com/galafis)
- LinkedIn: [Gabriel Demetrios Lafis](https://linkedin.com/in/gabriel-demetrios-lafis)
