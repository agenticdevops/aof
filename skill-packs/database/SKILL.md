# Database Operations Skill

You have expertise in database management, query optimization, and administration.

## PostgreSQL
- Connect: `psql -h <host> -U <user> -d <database>`
- Check connections: `SELECT pid, usename, application_name, state FROM pg_stat_activity;`
- Slow queries: `SELECT query, mean_exec_time FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;`
- Table sizes: `SELECT schemaname, tablename, pg_size_pretty(pg_total_relation_size(quote_ident(schemaname)||'.'||quote_ident(tablename))) FROM pg_tables ORDER BY pg_total_relation_size(quote_ident(schemaname)||'.'||quote_ident(tablename)) DESC;`
- Locks: `SELECT * FROM pg_locks l JOIN pg_stat_activity a ON l.pid = a.pid WHERE NOT granted;`

## Safety Practices
- Always run SELECT/EXPLAIN before UPDATE/DELETE
- Use transactions for multi-statement modifications: `BEGIN; ... ROLLBACK;` to test first
- VACUUM ANALYZE before and after large data operations
- Never run DDL (ALTER TABLE, DROP) without a backup or in a transaction
- Use `LIMIT` on unknown data sizes to avoid runaway queries

## Query Optimization
- Check query plan: `EXPLAIN (ANALYZE, BUFFERS) <query>;`
- Index usage: look for `Seq Scan` on large tables — potential missing index
- Index bloat: `SELECT * FROM pgstatindex('<index_name>');`
- Cache hit ratio: `SELECT sum(heap_blks_hit) / (sum(heap_blks_hit) + sum(heap_blks_read)) FROM pg_statio_user_tables;`
