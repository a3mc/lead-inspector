
# Solana Validator Leader Inspector

**Solana Validator Leader Inspector** is a Rust-based command-line tool designed to interact with the Solana blockchain. It provides insights into a validator's performance for a specified epoch or the current epoch, including the leader schedule, skipped leader slots, nearby leaders relative to the validator's assigned slots, and metrics such as prior leader latency and rank.

## Features

- Retrieve the current epoch and slot information.
- Query the leader schedule for a specific epoch or the current epoch.
- Validate Solana validator public keys.
- Calculate the first absolute slot of any given epoch.

## Installation

1. Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed on your system.
2. Clone the repository and navigate to the project directory:
   ```bash
   git clone <repository-url>
   cd lead-validator
   ```
3. Build the project:
   ```bash
   cargo build --release
   ```

## Usage

Run the program using the following command:

```bash
cargo run -- --validator <pubkey> --epoch <epoch>
```

### Parameters

- `--validator <pubkey>` (required): The public key of the Solana validator you want to query.
- `--epoch <epoch>` (optional): The epoch number to query. If not provided, the tool defaults to the current epoch.

### Examples

#### Query the Current Epoch

```bash
cargo run -- --validator B3aPj8cRWvBzkXxMRM2ZK8eqbG1DZW1mjt6eZgfebcYr
```

#### Query a Specific Epoch

```bash
cargo run -- --validator B3aPj8cRWvBzkXxMRM2ZK8eqbG1DZW1mjt6eZgfebcYr --epoch 250
```

## Output

The program provides the following details:

1. **Current Epoch and Slot Information**:
   - The current epoch and the first absolute slot of the queried epoch.
2. **Leader Schedule**:
   - Retrieves the leader schedule for the specified epoch.

### Example Output

```text
Using specified epoch: 250
First absolute slot of epoch 250: 20000000
Leader schedule for epoch 250 retrieved successfully.
```

## Development

### Dependencies

This program uses the following crates:

- `clap`: For command-line argument parsing.
- `tokio`: For asynchronous runtime.
- `anyhow`: For error handling.
- `solana-client`: To interact with the Solana RPC API.
- `solana-sdk`: For Solana-specific types and utilities.

### Build & Test

To build the project:
```bash
cargo build
```

To run tests:
```bash
cargo test
```

## Contributing

Contributions are welcome! Feel free to submit issues or pull requests to improve the project.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.

---

For any questions or issues, please reach out to the project maintainer.
