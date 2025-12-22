# ACP Daemon

HTTP REST API daemon for the AI Context Protocol.

## Installation

```bash
# Via cargo
cargo install acp-daemon

# Or via ACP CLI
acp install daemon
```

## Usage

```bash
# Start daemon in background
acpd start

# Start in foreground
acpd start --foreground
# or
acpd run

# Check status
acpd status

# Stop daemon
acpd stop
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/cache` | GET | Get full cache |
| `/config` | GET | Get configuration |
| `/vars` | GET | Get variables |
| `/symbols` | GET | List all symbols |
| `/symbols/{name}` | GET | Get symbol details |
| `/files` | GET | List all files |
| `/files/{path}` | GET | Get file details |
| `/domains` | GET | List domains |
| `/constraints` | GET | List constraints |

## Configuration

The daemon looks for ACP files in the project root:
- `.acp/acp.cache.json` - Indexed cache
- `.acp/acp.vars.json` - Variables
- `.acp.config.json` - Configuration

## License

MIT
