# browser-agent

MCP server for real-time browser automation via Chrome DevTools Protocol.

Gives Claude (or any MCP client) direct control of a local Chromium-based browser — navigate, read content, click, type, screenshot, extract contacts, press keys — all through the Model Context Protocol over stdio.

## Architecture

```
Claude ←[stdio/JSON-RPC]→ browser-agent ←[CDP/WebSocket]→ Edge/Chrome
```

- **Transport:** MCP stdio (JSON-RPC 2.0)
- **Browser control:** chromiumoxide 0.9 (CDP)
- **Runtime:** tokio async, single Page singleton via `Arc<Mutex<Option<Page>>>`
- **Error model:** all CDP errors → `Ok(CallToolResult::error)`, never `Err(ErrorData)`

## Tools

| Tool | Description |
|------|-------------|
| `ping` | Health check |
| `navigate(url)` | Open URL in browser |
| `get_content()` | Extract visible text (`document.body.innerText`, truncated at 100KB) |
| `click(selector)` | Click element by CSS selector |
| `click_at(x, y)` | Click at pixel coordinates |
| `type_text(selector, text)` | Focus element + type text |
| `screenshot()` | Capture viewport as base64 PNG |
| `find_contacts()` | Extract telegram handles, emails, phones, URLs from current page |
| `press_key(key)` | Press keyboard key (Enter, Tab, Escape, ...) |

## Build

```bash
cargo build --release
```

## Configuration

Add to `.mcp.json` (Claude Code / Claude Desktop):

```json
{
  "mcpServers": {
    "browser-agent": {
      "command": "/path/to/browser-agent",
      "args": []
    }
  }
}
```

Browser auto-launches on first `navigate()` call with `--remote-debugging-port=9222`.

Override browser path: `BROWSER_PATH=/path/to/chromium`

## Dependencies

- [rmcp](https://github.com/anthropics/rmcp) 0.16 — Rust MCP SDK
- [chromiumoxide](https://github.com/mattsse/chromiumoxide) 0.9 — CDP client
- tokio, serde, schemars, base64, futures, tracing

## License

MIT
