
<p align="center">
  <img src="assets/logo.png" width="240" alt="Poost logo" />
</p>

<p align="center"><b>Simple HTTP API for any Ere-compliant zkVM.</b></p>

Poost is an API wrapper on top of [Ere](https://github.com/eth-applied-research-group/ere) allowing you to execute, verify and create zkVM proofs by calling a HTTP endpoint.

---

## Table of Contents

1. [Features](#features)
2. [Quick Start](#quick-start)
3. [Manual Build](#manual-build)
4. [API](#api)
5. [Supported Backends](#supported-backends)
6. [Contributing](#contributing)
7. [License](#license)

---

## Features

* **REST API** for execution, proof generation, and verification.
* **Pluggable Backend Support**: Leverages `Ere` for backend integration.

---

## Quick Start

This example assumes you have Poost server running (e.g., via manual build or (TODO) a Docker image).

> The easiest way to start is by running the `test_workflow.sh` script.

## Manual Build

### Prerequisites

* **Rust** ≥ 1.80
* **zkVM Toolchain** Visit the respective zkVMs docs or checkout Ere on how to install.

```bash
# 1. Clone
git clone https://github.com/eth-applied-research-group/poost.git && cd poost

# 2. Run
cargo run --release
```

> Some zk‑VMs require **release** mode for proofs to finish in a reasonable time.

---

## API

The following endpoints are available:

| Endpoint   | Method | Purpose                                     |
|------------|--------|---------------------------------------------|
| `/info`    | `GET`  | Get server and system information           |
| `/execute` | `POST` | Run program and get execution metrics       |
| `/prove`   | `POST` | Generate proof for a program with inputs    |
| `/verify`  | `POST` | Verify a previously generated proof         |

## Supported Backends

Poost uses `Ere` for backend integration. Not all backends will be integrated, however since the API for Ere is the same for all, it is easy to add backends already supported by Ere.

## Contributing

Contributions are welcome!

## License

Dual‑licensed under **Apache‑2.0** and **MIT**. Choose either license at your discretion.
