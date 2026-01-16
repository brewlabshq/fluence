# Fluence

Automated Solana stake pool cranker that sends SOL to pool reserves and triggers update instructions. Supports Sanctum and native SPL stake pools.

## Quick Start

```bash
cp .env.example .env
# Edit .env with your configuration
cargo run --release
```

## Configuration

| Variable | Description |
|----------|-------------|
| `POOL_TYPE` | `sanctum` or `native` |
| `RPC_URL` | Solana RPC endpoint |
| `ADMIN_PRIVATE_KEY` | Base58-encoded private key |
| `POOL_RESERVE_ADDRESS` | Reserve address to send SOL |
| `POOL_ADDRESS` | Stake pool address (native pools only) |
| `CRANK_AMOUNT` | Amount in lamports |
| `EPOCH_POLL_INTERVAL` | How often to check for new epochs: `1m`, `5m`, `10m` |
| `EPOCH_STORAGE_TYPE` | `memory` or `file` (persist epoch state across restarts) |
| `EPOCH_STATE_FILE` | File path for epoch state (when using `file` storage) |
| `RUST_LOG` | Log level (e.g., `fluence=info`) |

## How It Works

- **Sanctum pools**: Sends SOL to reserve (auto-registered by Sanctum)
- **Native pools**: Sends SOL to reserve, then calls `UpdateStakePoolBalance`

The cranker runs once per epoch. It polls for epoch changes at the configured interval and persists state to avoid double-cranking.

## License

MIT
