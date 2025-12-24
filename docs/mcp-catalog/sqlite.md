---
sidebar_position: 5
sidebar_label: SQLite
---

# SQLite MCP Server

Query and interact with SQLite databases.

## Overview

| Property | Value |
|----------|-------|
| Package | `@modelcontextprotocol/server-sqlite` |
| Source | [GitHub](https://github.com/modelcontextprotocol/servers/tree/main/src/sqlite) |
| Transport | stdio |

## Installation

```bash
npx -y @modelcontextprotocol/server-sqlite /path/to/database.db
```

## Configuration

```yaml
mcp_servers:
  - name: sqlite
    command: npx
    args:
      - "-y"
      - "@modelcontextprotocol/server-sqlite"
      - "/data/app.db"
```

### In-Memory Database

```yaml
mcp_servers:
  - name: sqlite
    command: npx
    args:
      - "-y"
      - "@modelcontextprotocol/server-sqlite"
      - ":memory:"
```

## Tools

### read_query

Execute a read-only SQL query.

**Parameters**:
- `query` (string, required): SQL SELECT query

**Example**:
```json
{
  "tool": "read_query",
  "arguments": {
    "query": "SELECT * FROM users WHERE active = 1 LIMIT 10"
  }
}
```

### write_query

Execute a write SQL query (INSERT, UPDATE, DELETE).

**Parameters**:
- `query` (string, required): SQL write query

**Example**:
```json
{
  "tool": "write_query",
  "arguments": {
    "query": "UPDATE users SET last_login = datetime('now') WHERE id = 123"
  }
}
```

### create_table

Create a new table.

**Parameters**:
- `query` (string, required): CREATE TABLE statement

### list_tables

List all tables in the database.

**Parameters**: None

**Returns**: List of table names

### describe_table

Get schema of a table.

**Parameters**:
- `table_name` (string, required): Table name

**Returns**: Column definitions

### append_insight

Add an insight to the memo.

**Parameters**:
- `insight` (string, required): Insight text

## Resources

### Database Schema

```
sqlite:///path/to/database.db/schema
```

Returns full database schema.

### Memo

```
memo://insights
```

Accumulated insights and analysis notes.

## Use Cases

### Local Data Analyzer

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: data-analyzer
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: sqlite
      command: npx
      args: ["-y", "@modelcontextprotocol/server-sqlite", "/data/analytics.db"]
  system_prompt: |
    You analyze data in the SQLite database:
    1. Use list_tables to see available tables
    2. Use describe_table to understand schemas
    3. Use read_query to analyze data
    4. Use append_insight to record findings
```

### Log Database Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: log-db-agent
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: sqlite
      command: npx
      args: ["-y", "@modelcontextprotocol/server-sqlite", "/var/log/app.db"]
  system_prompt: |
    You analyze application logs stored in SQLite:
    - Find error patterns
    - Identify slow operations
    - Track user activity
    - Generate reports
```

### Config Database

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: config-manager
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: sqlite
      command: npx
      args: ["-y", "@modelcontextprotocol/server-sqlite", "/etc/app/config.db"]
  system_prompt: |
    You manage application configuration:
    - Read current settings
    - Update configuration values
    - Track configuration history
    - Validate settings
```

## Example Queries

### List Tables
```sql
SELECT name FROM sqlite_master WHERE type='table';
```

### Table Schema
```sql
PRAGMA table_info(users);
```

### Indexes
```sql
SELECT * FROM sqlite_master WHERE type='index';
```

### Database Size
```sql
SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size();
```

### Recent Records
```sql
SELECT * FROM logs ORDER BY timestamp DESC LIMIT 100;
```

### Aggregations
```sql
SELECT date(timestamp), count(*)
FROM events
GROUP BY date(timestamp)
ORDER BY 1 DESC;
```

## Write Operations

Unlike the PostgreSQL server, SQLite server allows write operations:

### Insert
```sql
INSERT INTO users (name, email) VALUES ('John', 'john@example.com');
```

### Update
```sql
UPDATE settings SET value = 'new_value' WHERE key = 'theme';
```

### Delete
```sql
DELETE FROM logs WHERE timestamp < date('now', '-30 days');
```

### Create Table
```sql
CREATE TABLE IF NOT EXISTS cache (
    key TEXT PRIMARY KEY,
    value TEXT,
    expires_at INTEGER
);
```

## Security

- **File Permissions**: Ensure database file has appropriate permissions
- **Backup**: SQLite databases should be backed up regularly
- **Locking**: SQLite uses file locking - only one writer at a time

### Read-Only Mode

For read-only access, use the `?mode=ro` parameter:

```yaml
mcp_servers:
  - name: sqlite
    command: npx
    args:
      - "-y"
      - "@modelcontextprotocol/server-sqlite"
      - "/data/app.db?mode=ro"
```

## Troubleshooting

### Database Locked

SQLite only allows one writer. Wait for other processes to finish.

### File Not Found

Ensure the path is absolute and the file exists:
```bash
ls -la /path/to/database.db
```

### Permission Denied

Check file permissions:
```bash
chmod 644 /path/to/database.db
```
