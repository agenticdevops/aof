---
sidebar_position: 2
sidebar_label: Filesystem
---

# Filesystem MCP Server

Read and write files on the local filesystem.

## Overview

| Property | Value |
|----------|-------|
| Package | `@modelcontextprotocol/server-filesystem` |
| Source | [GitHub](https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem) |
| Transport | stdio |

## Installation

```bash
npx -y @modelcontextprotocol/server-filesystem /path/to/allowed/directory
```

## Configuration

```yaml
mcp_servers:
  - name: filesystem
    command: npx
    args:
      - "-y"
      - "@modelcontextprotocol/server-filesystem"
      - "/home/user/projects"  # Allowed directory
```

### Multiple Directories

```yaml
mcp_servers:
  - name: filesystem
    command: npx
    args:
      - "-y"
      - "@modelcontextprotocol/server-filesystem"
      - "/home/user/projects"
      - "/var/log"
      - "/etc/app-config"
```

## Tools

### read_file

Read contents of a file.

**Parameters**:
- `path` (string, required): Path to file

**Example**:
```json
{
  "tool": "read_file",
  "arguments": {
    "path": "/home/user/projects/config.yaml"
  }
}
```

### read_multiple_files

Read multiple files at once.

**Parameters**:
- `paths` (array, required): List of file paths

**Example**:
```json
{
  "tool": "read_multiple_files",
  "arguments": {
    "paths": [
      "/home/user/projects/src/main.rs",
      "/home/user/projects/Cargo.toml"
    ]
  }
}
```

### write_file

Write content to a file.

**Parameters**:
- `path` (string, required): Path to file
- `content` (string, required): Content to write

**Example**:
```json
{
  "tool": "write_file",
  "arguments": {
    "path": "/home/user/projects/output.txt",
    "content": "Hello, World!"
  }
}
```

### create_directory

Create a new directory.

**Parameters**:
- `path` (string, required): Path to directory

### list_directory

List contents of a directory.

**Parameters**:
- `path` (string, required): Path to directory

**Example**:
```json
{
  "tool": "list_directory",
  "arguments": {
    "path": "/home/user/projects"
  }
}
```

### move_file

Move or rename a file.

**Parameters**:
- `source` (string, required): Source path
- `destination` (string, required): Destination path

### search_files

Search for files matching a pattern.

**Parameters**:
- `path` (string, required): Directory to search
- `pattern` (string, required): Glob pattern

**Example**:
```json
{
  "tool": "search_files",
  "arguments": {
    "path": "/home/user/projects",
    "pattern": "*.yaml"
  }
}
```

### get_file_info

Get metadata about a file.

**Parameters**:
- `path` (string, required): Path to file

## Resources

The filesystem server exposes files as resources:

```
file:///home/user/projects/config.yaml
```

## Use Cases

### Log Analysis Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: log-analyzer
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: filesystem
      command: npx
      args: ["-y", "@modelcontextprotocol/server-filesystem", "/var/log"]
  system_prompt: |
    You are a log analysis agent. Use the filesystem tools to read
    and analyze log files. Look for errors, patterns, and anomalies.
```

### Config Validator

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: config-validator
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: filesystem
      command: npx
      args: ["-y", "@modelcontextprotocol/server-filesystem", "/etc/myapp"]
  system_prompt: |
    You validate configuration files. Read configs from /etc/myapp
    and check for common issues, missing required fields, and
    security misconfigurations.
```

## Security

- **Path Restriction**: Only paths within allowed directories are accessible
- **Symlink Following**: Symlinks are followed but must resolve within allowed paths
- **No Escape**: Cannot access parent directories outside allowed paths

## Troubleshooting

### Permission Denied

Ensure the user running AOF has read/write permissions:

```bash
# Check permissions
ls -la /path/to/directory

# Fix permissions if needed
chmod -R 755 /path/to/directory
```

### Path Not Allowed

The path must be within the directories specified in the args:

```yaml
# This allows /home/user/projects and subdirectories
args: ["-y", "@modelcontextprotocol/server-filesystem", "/home/user/projects"]

# Trying to access /etc will fail
```
