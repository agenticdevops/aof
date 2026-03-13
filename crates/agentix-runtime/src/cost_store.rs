use std::sync::Arc;

use agentix_core::{AgentixError, CostRecord, CostSummary, RunCostSummary};
use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{Connection, params};

/// SQLite-backed cost tracking store.
///
/// One record per LLM call. Provides aggregation queries for per-run and
/// per-agent cost summaries. Uses COALESCE(actual_cost_usd, cost_usd) in
/// all aggregations to prefer provider-reported actual cost (COST-07).
pub struct CostStore {
    conn: Arc<Mutex<Connection>>,
}

impl CostStore {
    /// Open or create the SQLite database at the given path.
    ///
    /// Use `":memory:"` for tests (no file on disk).
    pub fn open(path: &str) -> Result<Self, AgentixError> {
        let conn = Connection::open(path).map_err(|e| {
            AgentixError::runtime(format!("Failed to open cost DB at '{path}': {e}"))
        })?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| AgentixError::runtime(format!("Failed to set WAL mode: {e}")))?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS cost_records (
                id              TEXT PRIMARY KEY,
                agent_name      TEXT NOT NULL,
                run_id          TEXT NOT NULL,
                model           TEXT NOT NULL,
                provider        TEXT NOT NULL,
                input_tokens    INTEGER NOT NULL DEFAULT 0,
                output_tokens   INTEGER NOT NULL DEFAULT 0,
                cost_usd        REAL NOT NULL DEFAULT 0.0,
                actual_cost_usd REAL,
                recorded_at     TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_cost_agent    ON cost_records(agent_name);
            CREATE INDEX IF NOT EXISTS idx_cost_run      ON cost_records(run_id);
            CREATE INDEX IF NOT EXISTS idx_cost_recorded ON cost_records(recorded_at DESC);
            ",
        )
        .map_err(|e| AgentixError::runtime(format!("Failed to create cost_records table: {e}")))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Insert a new cost record. Ignores duplicates (INSERT OR IGNORE).
    pub fn insert_record(&self, record: &CostRecord) -> Result<(), AgentixError> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR IGNORE INTO cost_records
             (id, agent_name, run_id, model, provider, input_tokens, output_tokens,
              cost_usd, actual_cost_usd, recorded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                record.id,
                record.agent_name,
                record.run_id,
                record.model,
                record.provider,
                record.input_tokens as i64,
                record.output_tokens as i64,
                record.cost_usd,
                record.actual_cost_usd,
                record.recorded_at.to_rfc3339(),
            ],
        )
        .map_err(|e| {
            AgentixError::runtime(format!("Failed to insert cost record '{}': {e}", record.id))
        })?;
        Ok(())
    }

    /// Aggregate all LLM calls in a run into a RunCostSummary.
    pub fn get_run_summary(&self, run_id: &str) -> Result<RunCostSummary, AgentixError> {
        let conn = self.conn.lock();
        let result = conn
            .query_row(
                "SELECT agent_name, model,
                        CAST(SUM(input_tokens) AS INTEGER),
                        CAST(SUM(output_tokens) AS INTEGER),
                        SUM(COALESCE(actual_cost_usd, cost_usd)),
                        MIN(recorded_at)
                 FROM cost_records
                 WHERE run_id = ?1
                 GROUP BY agent_name, model",
                params![run_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, f64>(4)?,
                        row.get::<_, String>(5)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| AgentixError::runtime(format!("Failed to query run summary: {e}")))?;

        match result {
            None => Err(AgentixError::runtime(format!(
                "No cost records for run '{run_id}'"
            ))),
            Some((agent_name, model, input_tokens, output_tokens, cost_usd, started_at_str)) => {
                let started_at = chrono::DateTime::parse_from_rfc3339(&started_at_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                Ok(RunCostSummary {
                    run_id: run_id.to_string(),
                    agent_name,
                    model,
                    input_tokens: input_tokens as u64,
                    output_tokens: output_tokens as u64,
                    cost_usd,
                    started_at,
                })
            }
        }
    }

    /// Aggregate all runs for an agent into a CostSummary.
    pub fn get_agent_summary(&self, agent_name: &str) -> Result<CostSummary, AgentixError> {
        let conn = self.conn.lock();
        let result = conn
            .query_row(
                "SELECT COUNT(DISTINCT run_id),
                        CAST(SUM(input_tokens) AS INTEGER),
                        CAST(SUM(output_tokens) AS INTEGER),
                        SUM(COALESCE(actual_cost_usd, cost_usd)),
                        MAX(recorded_at)
                 FROM cost_records
                 WHERE agent_name = ?1",
                params![agent_name],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, Option<f64>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                },
            )
            .map_err(|e| {
                AgentixError::runtime(format!("Failed to query agent cost summary: {e}"))
            })?;

        let (total_runs, total_input, total_output, total_cost, last_run_str) = result;
        let last_run_at = last_run_str.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|d| d.with_timezone(&Utc))
        });

        Ok(CostSummary {
            agent_name: agent_name.to_string(),
            total_runs: total_runs as u64,
            total_input_tokens: total_input as u64,
            total_output_tokens: total_output as u64,
            total_cost_usd: total_cost.unwrap_or(0.0),
            last_run_at,
        })
    }

    /// List per-run cost summaries for an agent, newest first.
    pub fn list_run_summaries(
        &self,
        agent_name: &str,
        limit: usize,
    ) -> Result<Vec<RunCostSummary>, AgentixError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT run_id, model,
                        CAST(SUM(input_tokens) AS INTEGER),
                        CAST(SUM(output_tokens) AS INTEGER),
                        SUM(COALESCE(actual_cost_usd, cost_usd)),
                        MIN(recorded_at)
                 FROM cost_records
                 WHERE agent_name = ?1
                 GROUP BY run_id, model
                 ORDER BY MIN(recorded_at) DESC
                 LIMIT ?2",
            )
            .map_err(|e| AgentixError::runtime(format!("Failed to prepare run list query: {e}")))?;

        let rows = stmt
            .query_map(params![agent_name, limit as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })
            .map_err(|e| AgentixError::runtime(format!("Failed to list run summaries: {e}")))?;

        let mut results = Vec::new();
        for row in rows {
            let (run_id, model, input_tokens, output_tokens, cost_usd, started_at_str) =
                row.map_err(|e| {
                    AgentixError::runtime(format!("Failed to read run summary row: {e}"))
                })?;
            let started_at = chrono::DateTime::parse_from_rfc3339(&started_at_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            results.push(RunCostSummary {
                run_id,
                agent_name: agent_name.to_string(),
                model,
                input_tokens: input_tokens as u64,
                output_tokens: output_tokens as u64,
                cost_usd,
                started_at,
            });
        }
        Ok(results)
    }

    /// List one CostSummary per agent across all agents, sorted by total_cost_usd DESC.
    pub fn list_all_summaries(&self) -> Result<Vec<CostSummary>, AgentixError> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT agent_name,
                        COUNT(DISTINCT run_id),
                        CAST(SUM(input_tokens) AS INTEGER),
                        CAST(SUM(output_tokens) AS INTEGER),
                        SUM(COALESCE(actual_cost_usd, cost_usd)),
                        MAX(recorded_at)
                 FROM cost_records
                 GROUP BY agent_name
                 ORDER BY SUM(COALESCE(actual_cost_usd, cost_usd)) DESC",
            )
            .map_err(|e| {
                AgentixError::runtime(format!("Failed to prepare all summaries query: {e}"))
            })?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Option<f64>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            })
            .map_err(|e| AgentixError::runtime(format!("Failed to list all summaries: {e}")))?;

        let mut results = Vec::new();
        for row in rows {
            let (agent_name, total_runs, total_input, total_output, total_cost, last_run_str) =
                row.map_err(|e| {
                    AgentixError::runtime(format!("Failed to read summary row: {e}"))
                })?;
            let last_run_at = last_run_str.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            });
            results.push(CostSummary {
                agent_name,
                total_runs: total_runs as u64,
                total_input_tokens: total_input as u64,
                total_output_tokens: total_output as u64,
                total_cost_usd: total_cost.unwrap_or(0.0),
                last_run_at,
            });
        }
        Ok(results)
    }

    /// Get total spend for an agent today (for daily budget check, COST-04).
    pub fn get_today_spend(&self, agent_name: &str) -> Result<f64, AgentixError> {
        let conn = self.conn.lock();
        let spend: Option<f64> = conn
            .query_row(
                "SELECT SUM(COALESCE(actual_cost_usd, cost_usd))
                 FROM cost_records
                 WHERE agent_name = ?1
                   AND date(recorded_at) = date('now')",
                params![agent_name],
                |row| row.get::<_, Option<f64>>(0),
            )
            .map_err(|e| AgentixError::runtime(format!("Failed to query today's spend: {e}")))?;
        Ok(spend.unwrap_or(0.0))
    }
}

// Use rusqlite's OptionalExtension for .optional()
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
