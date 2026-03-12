# Tools

Tools give your agent the ability to take actions in the world — running commands,
querying databases, calling APIs, and executing custom scripts.

## Tool Types

OpenAgentiX supports three tool types:

| Type | Description | Example |
|------|-------------|---------|
| `cli` | Execute a specific CLI binary | `kubectl`, `aws`, `psql` |
| `shell` | Execute a shell command or script | `echo {{msg}}`, `jq` pipes |
| `mcp` | Connect to an MCP server | Filesystem MCP, GitHub MCP |

## Defining Tools

Tools are defined as YAML files in your agent's `tools/` directory:

```
agents/my-agent/
├── agent.yaml
├── SOUL.md
└── tools/
    ├── kubectl.yaml
    ├── psql.yaml
    └── summarize.yaml
```

## CLI Tool

A CLI tool calls a specific binary with optional argument templates:

```yaml
# tools/kubectl.yaml
name: kubectl
type: cli
command: kubectl
description: Kubernetes cluster management
args:
  - "{{subcommand}}"
  - "-n"
  - "{{namespace}}"
```

When the LLM calls this tool with `{"subcommand": "get pods", "namespace": "default"}`,
it executes: `kubectl get pods -n default`

## Shell Tool

A shell tool executes arbitrary shell commands via `sh -c`:

```yaml
# tools/summarize.yaml
name: summarize
type: shell
command: "echo {{text}} | wc -w"
description: Count words in text
```

Shell tools support pipes, redirects, and multi-line scripts.

## Template Substitution

Both `cli` and `shell` tools support `{{variable}}` substitution in `command` and `args`.
The LLM provides values as JSON — strings are substituted directly; numbers and booleans
are JSON-serialized.

Unknown variables are left as-is (the placeholder string remains).

## Built-in CLI Tools

These common tools work with zero configuration (no tools/ YAML needed) when the binary
is on `$PATH`:
- `kubectl` — Kubernetes management
- `aws` — AWS CLI
- `psql` — PostgreSQL client
- `terraform` — Infrastructure as code
- `docker` — Container management
- `git` — Version control
- `sh` / `bash` — Shell scripts

## MCP Servers

MCP (Model Context Protocol) servers expose tools over a protocol connection. Configure them
in your agent's `mcp_servers` field or via the workspace config. See the [MCP guide](./mcp.md).

## Timeout

All CLI and shell tools have a 30-second default timeout. A tool that exceeds this limit
is killed and returns a timeout error as its observation. The agent then decides how to proceed.

## Error Handling

Tool failures do not abort the ReAct loop. The error message is returned as the tool's
observation, and the agent decides whether to retry, try a different approach, or give up.

Example observation on failure:
```
Tool 'kubectl' exited with code 1: Error from server (NotFound): pods "foo" not found
```
