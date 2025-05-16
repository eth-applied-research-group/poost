# Poost

A zero-knowledge proof service that provides an API for executing, proving, and verifying programs using different zkVM backends.

## Features

- REST API for ZK program execution, proof generation and proof verification
- Support for multiple ZK-VM backends (SP1, RISC0)

## Prerequisites

- Rust
- Specific zkVM toolchain

## Installation

1. Clone the repository:

```bash
git clone https://github.com/eth-applied-research-group/poost.git
cd poost
```

2. Build the project:

```bash
cargo build --release
```

> Note: Some zkVMs must be built in release mode in order for proofs to be generated in a reasonable amount of time.

> Note: Building the project does not install the zkVm specific compiler toolchains.

## Usage

### Starting the Server

Run the server with:

```bash
cargo run --release
```

The server will start on `http://localhost:3000` by default.

### Quick Testing

A test workflow script is provided to demonstrate the API usage:

```bash
./test_workflow.sh
```

## License

Apache and MIT

## Contributing

Contributions are welcome
