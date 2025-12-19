# Custom Tools

AOF supports extending agent capabilities through custom tools. You can create tools using MCP servers or by extending the built-in tool system.

## Creating MCP Tools

The easiest way to add custom tools is by creating an MCP server. MCP servers can be written in any language and communicate via stdio, SSE, or HTTP.

### Node.js MCP Server

Create a simple MCP server with the official SDK:

```bash
npm init -y
npm install @modelcontextprotocol/sdk
```

```javascript
// server.js
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';

const server = new Server({
  name: 'my-custom-tools',
  version: '1.0.0',
}, {
  capabilities: {
    tools: {},
  },
});

// Define a custom tool
server.setRequestHandler('tools/list', async () => ({
  tools: [
    {
      name: 'calculate_metrics',
      description: 'Calculate custom business metrics',
      inputSchema: {
        type: 'object',
        properties: {
          metric_name: {
            type: 'string',
            description: 'Name of the metric to calculate',
          },
          time_range: {
            type: 'string',
            description: 'Time range (e.g., "24h", "7d")',
          },
        },
        required: ['metric_name'],
      },
    },
  ],
}));

// Handle tool execution
server.setRequestHandler('tools/call', async (request) => {
  const { name, arguments: args } = request.params;

  if (name === 'calculate_metrics') {
    // Your custom logic here
    const result = await calculateMetrics(args.metric_name, args.time_range);
    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(result, null, 2),
        },
      ],
    };
  }

  throw new Error(`Unknown tool: ${name}`);
});

async function calculateMetrics(metricName, timeRange = '24h') {
  // Implement your metric calculation
  return {
    metric: metricName,
    value: Math.random() * 100,
    time_range: timeRange,
    timestamp: new Date().toISOString(),
  };
}

// Start the server
const transport = new StdioServerTransport();
await server.connect(transport);
```

### Using Your Custom Server

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: metrics-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a metrics analysis assistant.

  mcp_servers:
    - name: custom-metrics
      transport: stdio
      command: node
      args: ["./server.js"]
```

---

## Python MCP Server

```python
# server.py
import asyncio
import json
from mcp.server import Server
from mcp.server.stdio import stdio_server

app = Server("my-python-tools")

@app.list_tools()
async def list_tools():
    return [
        {
            "name": "analyze_logs",
            "description": "Analyze log files for patterns",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "log_path": {
                        "type": "string",
                        "description": "Path to log file"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "Pattern to search for"
                    }
                },
                "required": ["log_path"]
            }
        }
    ]

@app.call_tool()
async def call_tool(name: str, arguments: dict):
    if name == "analyze_logs":
        result = analyze_logs(arguments["log_path"], arguments.get("pattern"))
        return [{"type": "text", "text": json.dumps(result)}]
    raise ValueError(f"Unknown tool: {name}")

def analyze_logs(log_path: str, pattern: str = None):
    # Your analysis logic
    return {
        "log_path": log_path,
        "pattern": pattern,
        "matches": 42,
        "severity_breakdown": {
            "error": 5,
            "warning": 15,
            "info": 22
        }
    }

async def main():
    async with stdio_server() as (read, write):
        await app.run(read, write)

if __name__ == "__main__":
    asyncio.run(main())
```

```yaml
mcp_servers:
  - name: python-tools
    transport: stdio
    command: python
    args: ["./server.py"]
```

---

## Rust MCP Server

For high-performance tools, create an MCP server in Rust:

```rust
// Cargo.toml
// [dependencies]
// mcp-server = "0.1"
// tokio = { version = "1", features = ["full"] }
// serde_json = "1"

use mcp_server::{Server, Tool, ToolResult};
use serde_json::json;

#[tokio::main]
async fn main() {
    let server = Server::new("rust-tools", "1.0.0")
        .with_tool(Tool::new(
            "benchmark",
            "Run performance benchmarks",
            json!({
                "type": "object",
                "properties": {
                    "target": { "type": "string" },
                    "iterations": { "type": "number" }
                },
                "required": ["target"]
            }),
            |args| async move {
                let target = args["target"].as_str().unwrap();
                let iterations = args["iterations"].as_u64().unwrap_or(100);

                // Run benchmark
                let result = run_benchmark(target, iterations).await;

                ToolResult::success(json!({
                    "target": target,
                    "iterations": iterations,
                    "avg_ms": result.avg_ms,
                    "p99_ms": result.p99_ms
                }))
            },
        ));

    server.run_stdio().await;
}
```

---

## Tool Best Practices

### 1. Clear Descriptions

Write descriptions that help the LLM understand when to use the tool:

```javascript
{
  name: 'query_inventory',
  description: 'Query the inventory database for product availability. ' +
               'Use this when the user asks about stock levels, product counts, ' +
               'or warehouse inventory. Returns quantity and location data.',
  // ...
}
```

### 2. Comprehensive Input Schemas

Define all parameters with descriptions and constraints:

```javascript
{
  inputSchema: {
    type: 'object',
    properties: {
      product_id: {
        type: 'string',
        description: 'Product SKU or ID (e.g., "SKU-12345")',
        pattern: '^SKU-[0-9]+$'
      },
      warehouse: {
        type: 'string',
        description: 'Warehouse code',
        enum: ['US-EAST', 'US-WEST', 'EU-CENTRAL']
      },
      include_reserved: {
        type: 'boolean',
        description: 'Include reserved stock in count',
        default: false
      }
    },
    required: ['product_id']
  }
}
```

### 3. Structured Output

Return structured data the LLM can easily interpret:

```javascript
return {
  content: [
    {
      type: 'text',
      text: JSON.stringify({
        status: 'success',
        data: {
          product_id: 'SKU-12345',
          available: 150,
          reserved: 25,
          warehouses: [
            { code: 'US-EAST', quantity: 100 },
            { code: 'US-WEST', quantity: 50 }
          ]
        },
        metadata: {
          query_time_ms: 45,
          cache_hit: true
        }
      }, null, 2)
    }
  ]
};
```

### 4. Error Handling

Provide informative errors:

```javascript
try {
  const result = await performOperation(args);
  return { content: [{ type: 'text', text: JSON.stringify(result) }] };
} catch (error) {
  return {
    content: [
      {
        type: 'text',
        text: JSON.stringify({
          error: true,
          message: error.message,
          code: error.code || 'UNKNOWN_ERROR',
          suggestion: 'Check the input parameters and try again'
        })
      }
    ],
    isError: true
  };
}
```

### 5. Timeouts and Limits

Set appropriate timeouts for long-running operations:

```yaml
mcp_servers:
  - name: slow-tool
    transport: stdio
    command: node
    args: ["./slow-server.js"]
    timeout_secs: 300  # 5 minutes for data-intensive operations
```

---

## Testing Custom Tools

### Manual Testing

Use the MCP Inspector to test your server:

```bash
npx @modelcontextprotocol/inspector ./server.js
```

### Integration Testing

Test with a simple agent:

```yaml
# test-agent.yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: tool-tester
spec:
  model: google:gemini-2.5-flash
  instructions: |
    Test the custom tools by calling each one with sample inputs.
    Report the results.

  mcp_servers:
    - name: custom
      transport: stdio
      command: node
      args: ["./server.js"]
```

```bash
aofctl run agent test-agent.yaml -i "Test all available tools"
```

---

## See Also

- [MCP Integration](./mcp-integration.md) - Using MCP servers
- [Built-in Tools](./builtin-tools.md) - Native tool reference
- [MCP Specification](https://modelcontextprotocol.io/) - Official MCP documentation

