---
name: database-debug
description: "Database debugging and query execution"
homepage: "https://docs.aof.sh/skills/database-debug"
metadata:
  emoji: "🗄️"
  version: "1.0.0"
  requires:
    bins: []
    any_bins: ["psql", "mysql", "sqlite3"]
    env: []
    config: []
  tags: ["database", "debugging", "sql"]
---

# Database Debug Skill

Debug database issues and execute queries for diagnostics.

## When to Use This Skill

- Checking database connectivity
- Executing diagnostic queries
- Investigating slow queries
- Checking table structure
- Analyzing data

## Steps

1. **Connect to database** — `psql -h host -U user dbname`
2. **Check schema** — `\dt` or `SHOW TABLES;`
3. **Analyze query** — `EXPLAIN ANALYZE query;`
4. **Check locks** — `SELECT * FROM pg_locks;`
5. **Check size** — `SELECT pg_size_pretty(pg_total_relation_size(...));`
