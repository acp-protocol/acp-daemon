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

### Health & Status

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/stats` | GET | Cache statistics summary |

### Cache & Configuration

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/cache` | GET | Get full cache |
| `/config` | GET | Get configuration |
| `/vars` | GET | Get all variables |
| `/vars/{name}/expand` | GET | Expand a variable with context |

### Symbols

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/symbols` | GET | List all symbols |
| `/symbols/{name}` | GET | Get symbol details |
| `/callers/{symbol}` | GET | Get functions that call this symbol |
| `/callees/{symbol}` | GET | Get functions called by this symbol |

### Files

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/files` | GET | List all files |
| `/files/{path}` | GET | Get file details |

### Domains

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/domains` | GET | List all domains |
| `/domains/{name}` | GET | Get domain details |

### Constraints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/constraints/{path}` | GET | Get constraints for a file |

### Aggregate Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/map` | GET | Get project map overview |
| `/primer` | GET | Generate AI context primer |

## Configuration

The daemon looks for ACP files in the project root:
- `.acp/acp.cache.json` - Indexed cache
- `.acp/acp.vars.json` - Variables
- `.acp.config.json` - Configuration

## License

MIT
