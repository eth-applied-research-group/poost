# Poost

A zero-knowledge proof service that provides an API for executing, proving, and verifying programs using different zkVM backends.

## Features

- REST API for ZK program execution and verification
- Support for multiple ZK-VM backends (SP1, RISC0)
- Program execution with input parameters
- Proof generation and verification
- System information monitoring

## Prerequisites

- Rust (latest stable version)
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

## Usage

### Starting the Server

Run the server with:

```bash
cargo run --release
```

The server will start on `http://localhost:3000` by default.

### API Documentation

#### Available Endpoints

1. **Get System Information**

   ```http
   GET /info
   ```

   Returns information about the server's system resources and configuration.

2. **Execute Program**

   ```http
   POST /execute
   Content-Type: application/json

   {
     "program_id": "sp1",
     "input": {
       "value1": 10,
       "value2": 100
     }
   }
   ```

   Executes a program with the given inputs and returns execution metrics.

3. **Generate Proof**

   ```http
   POST /prove
   Content-Type: application/json

   {
     "program_id": "sp1",
     "input": {
       "value1": 10,
       "value2": 100
     }
   }
   ```

   Generates a zero-knowledge proof for program execution with given inputs.

4. **Verify Proof**

   ```http
   POST /verify
   Content-Type: application/json

   {
     "program_id": "sp1",
     "proof": "<proof-bytes>"
   }
   ```

   Verifies a previously generated zero-knowledge proof.

### Quick Testing

A test workflow script is provided to demonstrate the API usage:

```bash
./test_workflow.sh
```

## License

Apache and MIT

## Contributing

Contributions are welcome
