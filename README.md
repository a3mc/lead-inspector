# Validator Leader Inspector

Inspired by [slot-bandits](https://github.com/1000xsh/solana-slot-bandits)

A CLI tool written in Rust that provides detailed insights into a validator’s block production during a given epoch. It presents the leader schedule, identifies skipped leader slots, shows nearby leaders relative to the validator’s assigned slots, and summarizes metrics such as prior leader latency and ranking.

## Purpose

This tool offers clarity into observed slot skips, potential performance bottlenecks, and validator behavior under various conditions. Its structured, auditable implementation leverages reliable libraries, ensuring maintainability and ease of contribution.

## Features

- Retrieve the current epoch and slots information
- Query the leader schedule for a specific epoch or the current epoch
- Validate public keys
- Calculate the first absolute slot of any given epoch
- Check api.trillium.so/skip_blame for prior leader in list
- Check app.vx.tools for prior leader average latency and current TVC rank

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
cargo run -- --validator <validator_identity> --epoch <epoch_number>
```

### Parameters

- `--validator <validator_identity>` The public validator identity address you want to query
- `--epoch <epoch>` _(optional)_ The epoch number to query. If not provided, the tool defaults to the current epoch

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
   - The current epoch and the first absolute slot of the queried epoch
2. **Leader Schedule**:
   - Retrieves the leader schedule for the specified epoch.
3. **Skip Blame**:
   - Checks trillium.so for skip blame as potential skip reason from prior leader
4. **Latency and Rank**:
   - Checks app.vx.tools for the prior leaders average latency and current TVC rank

### Example Output

```text
Using configured epoch: 703
Validator GwHH8ciFhR8vejWCqmg8FWZUCNtubPY2esALvy5tBvji is assigned to 228 slots in epoch 703.
Using average slot duration: 0.400 seconds
[00:00:00] [░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 4/228 (52s)
[00:01:02] [███████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 44/228 (9m)                                                                
----------------------------------------
Block of slots: [303778624, 303778625, 303778626, 303778627] at approximately 2024-11-27 08:39:47.000 UTC
Previous Slot 303778623 Leader: 5ikB9XZNVsjwKb6hHT3FS3So1Z1SrDvU5yaniWEQyDEG ##ON BAD SKIP LIST## (Latency: 1.262132, Rank: 641)
Our Validator Slots 303778624 - 303778627: GwHH8ciFhR8vejWCqmg8FWZUCNtubPY2esALvy5tBvji
Next Slot 303778628 Leader: CpuDNi3iVoHXbaT8gHpzKe6rqeBasoYjEKi21q7NRVJS
Slot 303778625: no block produced (skipped?) or no leader info. Not produced by us.
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

For any questions or issues, please reach out to the project maintainer
