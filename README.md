# Fluence - Solana Stake Pool Cranker

A Rust-based automated cranker system that periodically sends SOL to stake pool reserves and triggers update instructions to boost APY. Supports both Sanctum-based and native SPL stake pools.

## Features

- **Dual LST Support**: Switch between Sanctum and Native SPL stake pools via environment configuration
- **Automated Scheduling**: Configurable interval-based cranking (e.g., hourly, daily)
- **Robust Error Handling**: Automatic retry logic with detailed logging
- **Simple Configuration**: Environment variable-based setup

## Architecture

```
Config → Scheduler → PoolHandler (Trait)
                          ├── SanctumPoolHandler
                          └── NativePoolHandler
```

## Installation

### Prerequisites

- Rust 1.75 or later
- Solana CLI (optional, for key generation)

### Build

```bash
cargo build --release
```

## Configuration

Create a `.env` file based on `.env.example`:

```bash
cp .env.example .env
```

### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `POOL_TYPE` | Pool type: "sanctum" or "native" | `sanctum` |
| `RPC_URL` | Solana RPC endpoint | `https://api.devnet.solana.com` |
| `ADMIN_PRIVATE_KEY` | Base58-encoded private key | `your_key_here` |
| `POOL_RESERVE_ADDRESS` | Reserve address to send SOL | `pubkey_here` |
| `POOL_ADDRESS` | Stake pool address (for native pools) | `pubkey_here` |
| `CRANK_AMOUNT` | Amount in lamports | `100000000` (0.1 SOL) |
| `CRANK_INTERVAL` | Crank interval | `1h`, `30m`, `1d` |
| `RUST_LOG` | Logging level | `fluence=info` |

### Duration Format

The `CRANK_INTERVAL` supports the following units:
- `s` - seconds (e.g., `30s`)
- `m` - minutes (e.g., `5m`)
- `h` - hours (e.g., `1h`)
- `d` - days (e.g., `1d`)

## Usage

### Running the Cranker

```bash
# Development
RUST_LOG=fluence=info cargo run

# Production
./target/release/fluence
```

### Testing on Devnet

1. Generate a new keypair or export existing one:
```bash
solana-keygen new -o ~/.config/solana/cranker.json
# Or export existing keypair to base58
```

2. Fund the wallet with devnet SOL:
```bash
solana airdrop 2 <your_pubkey> --url devnet
```

3. Configure `.env` with devnet settings:
```bash
POOL_TYPE=sanctum
RPC_URL=https://api.devnet.solana.com
ADMIN_PRIVATE_KEY=<your_base58_key>
POOL_RESERVE_ADDRESS=<devnet_pool_reserve>
CRANK_AMOUNT=100000000
CRANK_INTERVAL=5m
RUST_LOG=fluence=debug
```

4. Run the cranker:
```bash
cargo run
```

## How It Works

### Sanctum Pools

1. **Deposit**: Sends SOL to the reserve address via system transfer
2. **Crank**: Not required (deposits are auto-registered by Sanctum)

### Native SPL Stake Pools

1. **Deposit**: Sends SOL to the reserve stake account via system transfer
2. **Crank**: Calls `UpdateStakePoolBalance` instruction to register the deposit
   - Fetches stake pool account data
   - Builds permissionless update instruction
   - Sends transaction to update pool state

### Scheduling Loop

```rust
loop {
    interval.tick().await;

    // Execute crank cycle
    send_to_reserve() → confirm() → crank_pool()

    // Log results
    // Continue on errors
}
```

## Project Structure

```
fluence/
├── src/
│   ├── main.rs              # Entry point
│   ├── config.rs            # Configuration management
│   ├── error.rs             # Error types
│   ├── scheduler.rs         # Scheduling logic
│   ├── transaction/
│   │   └── mod.rs          # Transaction utilities
│   └── pool/
│       ├── mod.rs          # PoolHandler trait
│       ├── sanctum.rs      # Sanctum implementation
│       └── native.rs       # Native SPL implementation
├── Cargo.toml
├── .env.example
├── .gitignore
└── README.md
```

## Security

**IMPORTANT: Never commit your `.env` file or expose private keys!**

- Private keys are only loaded from environment variables
- Keys are never logged (only public keys for debugging)
- `.env` is in `.gitignore`
- Use secrets managers in production (AWS Secrets Manager, etc.)

## Error Handling

The cranker continues running even if individual crank cycles fail:

- **Configuration errors**: Fail fast at startup
- **Transient errors**: Retry with backoff (RPC, network)
- **Transaction errors**: Log and continue to next cycle
- **Fatal errors**: Shutdown gracefully

## Logging

```bash
# Info level (default)
RUST_LOG=fluence=info cargo run

# Debug level (verbose)
RUST_LOG=fluence=debug cargo run

# Error level only
RUST_LOG=fluence=error cargo run
```

## Development

### Running Tests

```bash
cargo test
```

### Building for Production

```bash
cargo build --release
```

The binary will be at `target/release/fluence`.

## Troubleshooting

### "Failed to parse keypair"

- Ensure your private key is base58-encoded
- Verify the key is 64 bytes when decoded
- Try generating a new keypair with `solana-keygen`

### "Invalid reserve address"

- Verify the address is a valid Solana pubkey
- Check that you're using the correct pool reserve address
- For native pools, ensure `POOL_ADDRESS` is also set

### "RPC error"

- Check your RPC endpoint is accessible
- Verify you're not hitting rate limits
- Consider using a paid RPC provider for production

### "Transaction error"

- Ensure admin wallet has sufficient SOL for transactions
- Check that the pool reserve address is correct
- Verify the pool is active and accepting deposits

## Future Enhancements

- [ ] Multiple pool support (parallel cranking)
- [ ] Dynamic amount calculation based on pool metrics
- [ ] Web dashboard for monitoring
- [ ] Prometheus metrics export
- [ ] Alert system (email/Slack/Discord)
- [ ] Health check endpoints
- [ ] Docker containerization

## License

This project is licensed under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
