---
sidebar_position: 8
sidebar_label: Puppeteer
---

# Puppeteer MCP Server

Browser automation for web scraping and testing.

## Overview

| Property | Value |
|----------|-------|
| Package | `@modelcontextprotocol/server-puppeteer` |
| Source | [GitHub](https://github.com/modelcontextprotocol/servers/tree/main/src/puppeteer) |
| Transport | stdio |

## Installation

```bash
npx -y @modelcontextprotocol/server-puppeteer
```

## Configuration

```yaml
mcp_servers:
  - name: puppeteer
    command: npx
    args: ["-y", "@modelcontextprotocol/server-puppeteer"]
```

### Headless Mode

```yaml
mcp_servers:
  - name: puppeteer
    command: npx
    args:
      - "-y"
      - "@modelcontextprotocol/server-puppeteer"
```

The server runs in headless mode by default, suitable for server environments.

## Tools

### puppeteer_navigate

Navigate to a URL.

**Parameters**:
- `url` (string, required): URL to navigate to

**Example**:
```json
{
  "tool": "puppeteer_navigate",
  "arguments": {
    "url": "https://example.com"
  }
}
```

### puppeteer_screenshot

Take a screenshot of the current page.

**Parameters**:
- `name` (string, required): Screenshot name
- `selector` (string, optional): CSS selector to screenshot
- `width` (number, optional): Viewport width
- `height` (number, optional): Viewport height

**Example**:
```json
{
  "tool": "puppeteer_screenshot",
  "arguments": {
    "name": "homepage",
    "width": 1280,
    "height": 720
  }
}
```

### puppeteer_click

Click an element.

**Parameters**:
- `selector` (string, required): CSS selector

**Example**:
```json
{
  "tool": "puppeteer_click",
  "arguments": {
    "selector": "#login-button"
  }
}
```

### puppeteer_fill

Fill a form field.

**Parameters**:
- `selector` (string, required): CSS selector
- `value` (string, required): Value to fill

**Example**:
```json
{
  "tool": "puppeteer_fill",
  "arguments": {
    "selector": "#username",
    "value": "testuser"
  }
}
```

### puppeteer_select

Select an option from a dropdown.

**Parameters**:
- `selector` (string, required): CSS selector
- `value` (string, required): Option value

### puppeteer_hover

Hover over an element.

**Parameters**:
- `selector` (string, required): CSS selector

### puppeteer_evaluate

Execute JavaScript in the page context.

**Parameters**:
- `script` (string, required): JavaScript code

**Example**:
```json
{
  "tool": "puppeteer_evaluate",
  "arguments": {
    "script": "document.title"
  }
}
```

## Resources

### Console Logs

```
console://logs
```

Returns browser console output.

### Screenshots

```
screenshot://<name>
```

Returns a previously taken screenshot.

## Use Cases

### Web Scraper Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: web-scraper
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: puppeteer
      command: npx
      args: ["-y", "@modelcontextprotocol/server-puppeteer"]
  system_prompt: |
    You scrape web pages for data:
    1. Navigate to target pages
    2. Wait for dynamic content
    3. Extract required information
    4. Handle pagination

    Always be respectful of robots.txt and rate limits.
```

### UI Testing Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: ui-tester
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: puppeteer
      command: npx
      args: ["-y", "@modelcontextprotocol/server-puppeteer"]
  system_prompt: |
    You perform UI testing:
    - Navigate through user flows
    - Fill forms and submit
    - Verify page content
    - Take screenshots of results
    - Report visual regressions
```

### Login Automation Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: login-tester
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: puppeteer
      command: npx
      args: ["-y", "@modelcontextprotocol/server-puppeteer"]
  system_prompt: |
    You test login flows:
    1. Navigate to login page
    2. Fill username and password
    3. Click login button
    4. Verify successful login
    5. Check for error messages

    Test with provided test credentials only.
```

### Visual Regression Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: visual-regression
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: puppeteer
      command: npx
      args: ["-y", "@modelcontextprotocol/server-puppeteer"]
  system_prompt: |
    You detect visual regressions:
    - Navigate to key pages
    - Take screenshots at various viewports
    - Compare with baseline images
    - Report visual differences
    - Document UI changes
```

## Example Workflows

### Form Submission

```javascript
// 1. Navigate to form
puppeteer_navigate({ url: "https://example.com/form" })

// 2. Fill fields
puppeteer_fill({ selector: "#name", value: "John Doe" })
puppeteer_fill({ selector: "#email", value: "john@example.com" })

// 3. Select option
puppeteer_select({ selector: "#country", value: "US" })

// 4. Submit
puppeteer_click({ selector: "#submit" })

// 5. Screenshot result
puppeteer_screenshot({ name: "form-submitted" })
```

### Data Extraction

```javascript
// 1. Navigate
puppeteer_navigate({ url: "https://example.com/data" })

// 2. Wait for content (via JavaScript)
puppeteer_evaluate({
  script: "await new Promise(r => setTimeout(r, 2000))"
})

// 3. Extract data
puppeteer_evaluate({
  script: `
    Array.from(document.querySelectorAll('.item'))
      .map(el => ({
        title: el.querySelector('.title').textContent,
        price: el.querySelector('.price').textContent
      }))
  `
})
```

## Selectors

### CSS Selectors

```javascript
// By ID
puppeteer_click({ selector: "#my-id" })

// By class
puppeteer_click({ selector: ".my-class" })

// By attribute
puppeteer_click({ selector: "[data-testid='submit']" })

// Complex
puppeteer_click({ selector: "form.login button[type='submit']" })
```

### Best Practices

1. **Use data-testid**: `[data-testid='login-button']`
2. **Avoid brittle selectors**: Don't rely on class names that may change
3. **Be specific**: Use unique identifiers when possible

## Troubleshooting

### Element Not Found

Wait for element to appear:
```javascript
puppeteer_evaluate({
  script: `
    await page.waitForSelector('#element', { timeout: 5000 })
  `
})
```

### Page Not Loaded

Wait for navigation:
```javascript
puppeteer_navigate({ url: "https://example.com" })
puppeteer_evaluate({
  script: "await page.waitForNavigation()"
})
```

### Timeout Errors

Increase timeout or check network:
- Page may be slow to load
- Check if site blocks automated browsers
- Verify network connectivity

### Chrome/Chromium Not Found

Install Chrome or Chromium:
```bash
# Ubuntu/Debian
apt-get install chromium-browser

# macOS
brew install chromium
```

## Security Considerations

- **Sandboxed**: Browser runs in sandboxed mode
- **No Persistent Storage**: Cookies/storage cleared between sessions
- **Resource Limits**: Memory and CPU usage monitored
- **Network Isolation**: Consider firewall rules for target sites
