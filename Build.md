# Building the Application

This project uses a combination of SvelteKit (Frontend) and Tauri/Rust (Backend) with a custom SQLite encryption driver (SQLCipher). To ensure cross-platform reproducibility, we strictly manage all developer dependencies using [`mise`](https://mise.jdx.dev/).

## 1. Prerequisites

Before you begin, ensure you have the following installed on your system:

- **Git**
- **mise** (The dev tool manager). See [installation instructions](https://mise.jdx.dev/getting-started.html).

You do **NOT** need to install Node.js, pnpm, or Rust globally. `mise` will manage the exact required versions for this project.

## 2. Environment Setup

Clone the repository and install the required tools using `mise`:

```bash
git clone <repository_url> personal
cd personal
mise install
```

This command automatically downloads and configures:

- **Node.js** (LTS)
- **pnpm** (Latest)
- **Rust** (stable)

### System Requirements (SQLCipher / OpenSSL)

Because we use the `bundled-sqlcipher` driver for encrypted SQLite, you must have OpenSSL or LibreSSL installed on your system to provide `libcrypto`.

- **Windows**: Install OpenSSL using `vcpkg install openssl:x64-windows`, `choco install openssl`, or any other preferred method.
- **macOS / Linux**: Usually pre-installed or available via your package manager (e.g. `brew install openssl` or `apt install libssl-dev`).

## 3. Installing Dependencies

Once `mise install` completes, install the project's JavaScript dependencies:

```bash
pnpm install
```

## 4. Development Workflow

To start the application in development mode (which spins up both the Vite frontend server and the Tauri backend watcher):

```bash
pnpm tauri dev
```

### Validation Commands

Before committing any code, ensure you run the validation checks (as required by `AGENTS.md`):

```bash
# Frontend validation
pnpm check      # Svelte type checking
pnpm lint       # ESLint rules
pnpm format     # Prettier formatting

# Backend validation
cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## 5. Production Build

To build a standalone production binary:

```bash
pnpm tauri build
```

The compiled binaries will be located in `src-tauri/target/release/`.
