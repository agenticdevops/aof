---
sidebar_position: 7
sidebar_label: Fetch
---

# Fetch MCP Server

Make HTTP requests and fetch web content.

## Overview

| Property | Value |
|----------|-------|
| Package | `@modelcontextprotocol/server-fetch` |
| Source | [GitHub](https://github.com/modelcontextprotocol/servers/tree/main/src/fetch) |
| Transport | stdio |

## Installation

```bash
npx -y @modelcontextprotocol/server-fetch
```

## Configuration

```yaml
mcp_servers:
  - name: fetch
    command: npx
    args: ["-y", "@modelcontextprotocol/server-fetch"]
```

### With Custom User Agent

```yaml
mcp_servers:
  - name: fetch
    command: npx
    args:
      - "-y"
      - "@modelcontextprotocol/server-fetch"
      - "--user-agent"
      - "AOF-Agent/1.0"
```

## Tools

### fetch

Fetch content from a URL.

**Parameters**:
- `url` (string, required): URL to fetch
- `max_length` (number, optional): Maximum content length to return
- `start_index` (number, optional): Start position for pagination
- `raw` (boolean, optional): Return raw HTML instead of markdown

**Example**:
```json
{
  "tool": "fetch",
  "arguments": {
    "url": "https://api.example.com/status",
    "max_length": 5000
  }
}
```

**Returns**:
- For HTML pages: Converted markdown content
- For JSON APIs: Raw JSON response
- For other content: Raw text

## Features

### Automatic Content Conversion

The fetch server automatically:
- Converts HTML to readable markdown
- Preserves JSON structure
- Extracts text from common formats
- Handles character encoding

### Pagination Support

For large content, use pagination:

```json
{
  "tool": "fetch",
  "arguments": {
    "url": "https://example.com/long-page",
    "max_length": 5000,
    "start_index": 0
  }
}
```

Then fetch the next chunk:

```json
{
  "tool": "fetch",
  "arguments": {
    "url": "https://example.com/long-page",
    "max_length": 5000,
    "start_index": 5000
  }
}
```

### Raw HTML Mode

Get raw HTML when needed:

```json
{
  "tool": "fetch",
  "arguments": {
    "url": "https://example.com/page",
    "raw": true
  }
}
```

## Use Cases

### API Monitor Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: api-monitor
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: fetch
      command: npx
      args: ["-y", "@modelcontextprotocol/server-fetch"]
  system_prompt: |
    You monitor API endpoints for health:
    1. Fetch status endpoints
    2. Check response codes
    3. Validate JSON structure
    4. Report anomalies

    Endpoints to monitor:
    - https://api.example.com/health
    - https://api.example.com/v1/status
```

### Documentation Fetcher

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: doc-fetcher
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: fetch
      command: npx
      args: ["-y", "@modelcontextprotocol/server-fetch"]
  system_prompt: |
    You fetch and summarize documentation:
    - Retrieve documentation pages
    - Extract key information
    - Summarize for users
    - Find relevant sections
```

### Price Tracker

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: price-tracker
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: fetch
      command: npx
      args: ["-y", "@modelcontextprotocol/server-fetch"]
  system_prompt: |
    You track prices from APIs:
    - Fetch pricing data
    - Compare historical prices
    - Alert on significant changes
    - Generate price reports
```

### Webhook Tester

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: webhook-tester
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: fetch
      command: npx
      args: ["-y", "@modelcontextprotocol/server-fetch"]
  system_prompt: |
    You test webhook endpoints:
    - Verify endpoint accessibility
    - Check response formats
    - Validate SSL certificates
    - Test authentication flows
```

## Content Type Handling

| Content Type | Handling |
|--------------|----------|
| `text/html` | Converted to markdown |
| `application/json` | Returned as-is |
| `text/plain` | Returned as-is |
| `text/markdown` | Returned as-is |
| Other | Best-effort text extraction |

## Headers

The server sets default headers:
- `User-Agent`: Configurable via args
- `Accept`: `text/html,application/json,text/plain,*/*`

## Limitations

- **No POST/PUT/DELETE**: Only GET requests
- **No Custom Headers**: Cannot set auth headers per-request
- **No Cookies**: Stateless requests only
- **Size Limits**: Large responses are truncated

## Security

- **URL Validation**: URLs are validated before fetching
- **No Local Files**: Cannot fetch `file://` URLs
- **Timeout**: Requests timeout after 30 seconds
- **HTTPS Preferred**: HTTP allowed but HTTPS recommended

## Troubleshooting

### Connection Refused

The target server may be blocking requests:
- Try with a different user agent
- Check if the site requires authentication
- Verify the URL is accessible

### Timeout

Large pages may timeout:
- Use `max_length` to limit response size
- Try a more specific URL path
- Check network connectivity

### Empty Response

Some sites return empty for bots:
- Site may require JavaScript rendering (use puppeteer instead)
- Site may block automated requests
- Check robots.txt compliance

### SSL Errors

Certificate validation issues:
- Verify the site's SSL certificate is valid
- Check for certificate chain issues
