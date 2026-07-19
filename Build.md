# Building the Application

This project uses a pure Rust stack with `iced` for the frontend and a custom SQLite encryption driver (SQLCipher) for the backend database.

## 1. Prerequisites

Before you begin, ensure you have the following installed on your system:

- **Git**
- **Rust** (stable, via rustup)

### System Requirements (SQLCipher / OpenSSL)

Because we use the `bundled-sqlcipher` driver for encrypted SQLite, you must have OpenSSL or LibreSSL installed on your system to provide `libcrypto`.

- **Windows**: Install OpenSSL using `vcpkg install openssl:x64-windows`, `choco install openssl`, or any other preferred method.
- **macOS / Linux**: Usually pre-installed or available via your package manager (e.g. `brew install openssl` or `apt install libssl-dev`).

## 2. Environment Setup

Clone the repository:

```bash
git clone https://github.com/kitswas/personal.git
cd personal
```

## 3. Development Workflow

To start the application in development mode:

```bash
cargo run
```

### Validation Commands

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## 4. Production Build

To build a standalone production binary:

```bash
cargo build --release
```

The compiled binaries will be located in `target/release/`.
