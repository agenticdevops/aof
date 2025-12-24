---
sidebar_position: 4
sidebar_label: PostgreSQL
---

# PostgreSQL MCP Server

Query and interact with PostgreSQL databases.

## Overview

| Property | Value |
|----------|-------|
| Package | `@modelcontextprotocol/server-postgres` |
| Source | [GitHub](https://github.com/modelcontextprotocol/servers/tree/main/src/postgres) |
| Transport | stdio |

## Installation

```bash
npx -y @modelcontextprotocol/server-postgres postgresql://user:pass@host:5432/db
```

## Configuration

```yaml
mcp_servers:
  - name: postgres
    command: npx
    args:
      - "-y"
      - "@modelcontextprotocol/server-postgres"
      - "${DATABASE_URL}"
```

### Connection String Format

```
postgresql://[user[:password]@][host][:port][/database][?options]
```

Examples:
```bash
# Local database
postgresql://localhost/mydb

# With credentials
postgresql://user:password@localhost:5432/mydb

# With SSL
postgresql://user:password@host:5432/mydb?sslmode=require
```

## Tools

### query

Execute a read-only SQL query.

**Parameters**:
- `sql` (string, required): SQL query to execute

**Example**:
```json
{
  "tool": "query",
  "arguments": {
    "sql": "SELECT * FROM users WHERE status = 'active' LIMIT 10"
  }
}
```

**Returns**: Query results as JSON array

## Resources

The server exposes database schema as resources:

### Schema Resource

```
postgres://host/database/schema
```

Returns table definitions, columns, and relationships.

## Use Cases

### Database Analyst Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: db-analyst
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: postgres
      command: npx
      args: ["-y", "@modelcontextprotocol/server-postgres", "${DATABASE_URL}"]
  system_prompt: |
    You are a database analyst. Help users:
    - Write and optimize SQL queries
    - Analyze data patterns
    - Find anomalies in data
    - Generate reports

    Always use read-only queries. Never modify data.
```

### Schema Inspector

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: schema-inspector
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: postgres
      command: npx
      args: ["-y", "@modelcontextprotocol/server-postgres", "${DATABASE_URL}"]
  system_prompt: |
    You analyze database schemas:
    - Identify missing indexes
    - Find normalization issues
    - Suggest performance improvements
    - Document table relationships

    Use queries like:
    - SELECT * FROM information_schema.tables
    - SELECT * FROM information_schema.columns
    - SELECT * FROM pg_indexes
```

### Metrics Collector

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: db-metrics
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: postgres
      command: npx
      args: ["-y", "@modelcontextprotocol/server-postgres", "${DATABASE_URL}"]
  system_prompt: |
    You collect database metrics for monitoring:
    - Active connections
    - Slow queries
    - Table sizes
    - Index usage
    - Lock contention

    Query pg_stat_* views for metrics.
```

## Example Queries

### Table Information
```sql
SELECT table_name, table_type
FROM information_schema.tables
WHERE table_schema = 'public';
```

### Column Details
```sql
SELECT column_name, data_type, is_nullable, column_default
FROM information_schema.columns
WHERE table_name = 'users';
```

### Index Usage
```sql
SELECT schemaname, tablename, indexname, idx_scan, idx_tup_read
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;
```

### Slow Queries
```sql
SELECT query, calls, mean_time, total_time
FROM pg_stat_statements
ORDER BY mean_time DESC
LIMIT 10;
```

### Table Sizes
```sql
SELECT relname, pg_size_pretty(pg_total_relation_size(relid))
FROM pg_catalog.pg_statio_user_tables
ORDER BY pg_total_relation_size(relid) DESC;
```

### Active Connections
```sql
SELECT count(*), state, usename, application_name
FROM pg_stat_activity
GROUP BY state, usename, application_name;
```

## Security

- **Read-Only**: The server only allows SELECT queries
- **No DDL**: Cannot CREATE, ALTER, or DROP
- **No DML**: Cannot INSERT, UPDATE, or DELETE
- **Connection Security**: Use SSL for production databases

### Recommended Setup

```yaml
# Use a read-only database user
mcp_servers:
  - name: postgres
    command: npx
    args:
      - "-y"
      - "@modelcontextprotocol/server-postgres"
      - "postgresql://readonly_user:${DB_PASSWORD}@host:5432/mydb?sslmode=require"
```

Create a read-only user in PostgreSQL:
```sql
CREATE USER readonly_user WITH PASSWORD 'secret';
GRANT CONNECT ON DATABASE mydb TO readonly_user;
GRANT USAGE ON SCHEMA public TO readonly_user;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO readonly_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO readonly_user;
```

## Troubleshooting

### Connection Refused

Check database is accessible:
```bash
psql postgresql://user:pass@host:5432/db
```

### SSL Required

Add SSL mode to connection string:
```
postgresql://user:pass@host:5432/db?sslmode=require
```

### Permission Denied

Ensure user has SELECT permissions:
```sql
GRANT SELECT ON ALL TABLES IN SCHEMA public TO your_user;
```
