//! Benchmark runner and report generator

use super::*;
use std::fs;
use std::path::Path;

/// Run all benchmarks and generate report
pub fn run_full_benchmark() -> BenchmarkReport {
    let config = BenchmarkConfig::default();
    let scenarios = get_benchmark_scenarios();

    let mut results = Vec::new();

    println!("Running token efficiency benchmarks...\n");

    for scenario in &scenarios {
        println!("Scenario: {}", scenario.name);
        println!("  {}", scenario.description);

        let (unified, per_op) = compare_approaches(scenario, &config);

        println!("  Unified:  {} tokens ({}% success)",
            unified.total_tokens,
            (unified.success_rate * 100.0) as u32
        );
        println!("  Per-Op:   {} tokens ({}% success)",
            per_op.total_tokens,
            (per_op.success_rate * 100.0) as u32
        );

        let winner = if unified.total_tokens < per_op.total_tokens {
            "Unified"
        } else {
            "Per-Op"
        };
        let diff_pct = ((per_op.total_tokens as f64 / unified.total_tokens as f64) - 1.0) * 100.0;

        println!("  Winner:   {} ({:+.1}%)\n", winner, -diff_pct);

        results.push((unified, per_op));
    }

    BenchmarkReport {
        scenarios: scenarios.clone(),
        results,
        config,
    }
}

/// Full benchmark report
#[derive(Debug, Clone)]
pub struct BenchmarkReport {
    pub scenarios: Vec<BenchmarkScenario>,
    pub results: Vec<(BenchmarkResult, BenchmarkResult)>,
    pub config: BenchmarkConfig,
}

impl BenchmarkReport {
    /// Generate markdown report
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("# Token Efficiency Benchmark Report\n\n");
        md.push_str(&format!("**Generated:** {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        md.push_str("## Configuration\n\n");
        md.push_str(&format!("- Simulate retries: {}\n", self.config.simulate_retries));
        md.push_str(&format!("- Unified retry rate: {:.1}%\n", self.config.unified_retry_rate * 100.0));
        md.push_str(&format!("- Per-op retry rate: {:.1}%\n", self.config.per_op_retry_rate * 100.0));
        md.push_str(&format!("- Simulate parallel: {}\n\n", self.config.simulate_parallel));

        md.push_str("## Summary\n\n");
        md.push_str("| Scenario | Tasks | Unified | Per-Op | Winner | Difference |\n");
        md.push_str("|----------|-------|---------|--------|--------|------------|\n");

        for (scenario, (unified, per_op)) in self.scenarios.iter().zip(&self.results) {
            let winner = if unified.total_tokens < per_op.total_tokens {
                "✅ Unified"
            } else {
                "✅ Per-Op"
            };
            let diff_pct = ((per_op.total_tokens as f64 / unified.total_tokens as f64) - 1.0) * 100.0;

            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {:+.1}% |\n",
                scenario.name,
                scenario.tasks.len(),
                unified.total_tokens,
                per_op.total_tokens,
                winner,
                -diff_pct
            ));
        }

        md.push_str("\n## Detailed Results\n\n");

        for (i, (scenario, (unified, per_op))) in self.scenarios.iter().zip(&self.results).enumerate() {
            md.push_str(&format!("### {}. {}\n\n", i + 1, scenario.name));
            md.push_str(&format!("**Description:** {}\n\n", scenario.description));
            md.push_str(&format!("**Tasks:** {}\n\n", scenario.tasks.len()));

            md.push_str("#### Token Breakdown\n\n");
            md.push_str("| Component | Unified | Per-Op | Difference |\n");
            md.push_str("|-----------|---------|--------|------------|\n");

            md.push_str(&format!(
                "| Tool Definitions | {} | {} | {:+} |\n",
                unified.breakdown.tool_definitions,
                per_op.breakdown.tool_definitions,
                per_op.breakdown.tool_definitions as i32 - unified.breakdown.tool_definitions as i32
            ));

            md.push_str(&format!(
                "| System Prompt | {} | {} | {:+} |\n",
                unified.breakdown.system_prompt,
                per_op.breakdown.system_prompt,
                per_op.breakdown.system_prompt as i32 - unified.breakdown.system_prompt as i32
            ));

            md.push_str(&format!(
                "| Reasoning | {} | {} | {:+} |\n",
                unified.breakdown.reasoning,
                per_op.breakdown.reasoning,
                per_op.breakdown.reasoning as i32 - unified.breakdown.reasoning as i32
            ));

            md.push_str(&format!(
                "| Tool Calls | {} | {} | {:+} |\n",
                unified.breakdown.tool_calls,
                per_op.breakdown.tool_calls,
                per_op.breakdown.tool_calls as i32 - unified.breakdown.tool_calls as i32
            ));

            md.push_str(&format!(
                "| Tool Outputs | {} | {} | {:+} |\n",
                unified.breakdown.tool_outputs,
                per_op.breakdown.tool_outputs,
                per_op.breakdown.tool_outputs as i32 - unified.breakdown.tool_outputs as i32
            ));

            md.push_str(&format!(
                "| LLM Responses | {} | {} | {:+} |\n",
                unified.breakdown.llm_responses,
                per_op.breakdown.llm_responses,
                per_op.breakdown.llm_responses as i32 - unified.breakdown.llm_responses as i32
            ));

            md.push_str(&format!(
                "| Retry Overhead | {} | {} | {:+} |\n",
                unified.breakdown.retry_overhead,
                per_op.breakdown.retry_overhead,
                per_op.breakdown.retry_overhead as i32 - unified.breakdown.retry_overhead as i32
            ));

            md.push_str(&format!(
                "| **TOTAL** | **{}** | **{}** | **{:+}** |\n\n",
                unified.total_tokens,
                per_op.total_tokens,
                per_op.total_tokens as i32 - unified.total_tokens as i32
            ));

            md.push_str("#### Execution Metrics\n\n");
            md.push_str("| Metric | Unified | Per-Op |\n");
            md.push_str("|--------|---------|--------|\n");

            md.push_str(&format!(
                "| Success Rate | {:.1}% | {:.1}% |\n",
                unified.success_rate * 100.0,
                per_op.success_rate * 100.0
            ));

            md.push_str(&format!(
                "| Retries | {} | {} |\n",
                unified.metrics.retries,
                per_op.metrics.retries
            ));

            md.push_str(&format!(
                "| Errors | {} | {} |\n",
                unified.metrics.errors,
                per_op.metrics.errors
            ));

            md.push_str(&format!(
                "| Parallel Calls | {} | {} |\n\n",
                unified.metrics.parallel_calls,
                per_op.metrics.parallel_calls
            ));
        }

        md.push_str("## Analysis\n\n");
        md.push_str(&self.generate_analysis());

        md.push_str("\n## Recommendations\n\n");
        md.push_str(&self.generate_recommendations());

        md
    }

    fn generate_analysis(&self) -> String {
        let mut analysis = String::new();

        // Calculate overall statistics
        let total_unified: usize = self.results.iter().map(|(u, _)| u.total_tokens).sum();
        let total_per_op: usize = self.results.iter().map(|(_, p)| p.total_tokens).sum();

        let unified_wins = self.results.iter().filter(|(u, p)| u.total_tokens < p.total_tokens).count();
        let per_op_wins = self.results.len() - unified_wins;

        analysis.push_str(&format!("### Overall Statistics\n\n"));
        analysis.push_str(&format!("- **Total tokens (all scenarios):**\n"));
        analysis.push_str(&format!("  - Unified: {} tokens\n", total_unified));
        analysis.push_str(&format!("  - Per-Op: {} tokens\n", total_per_op));
        analysis.push_str(&format!("  - Difference: {:+} tokens ({:+.1}%)\n\n",
            total_per_op as i32 - total_unified as i32,
            ((total_per_op as f64 / total_unified as f64) - 1.0) * 100.0
        ));

        analysis.push_str(&format!("- **Scenarios won:**\n"));
        analysis.push_str(&format!("  - Unified: {}/{}\n", unified_wins, self.results.len()));
        analysis.push_str(&format!("  - Per-Op: {}/{}\n\n", per_op_wins, self.results.len()));

        analysis.push_str("### Key Findings\n\n");

        // Simple task analysis
        if let Some((unified, per_op)) = self.results.first() {
            let diff = per_op.total_tokens as i32 - unified.total_tokens as i32;
            let pct = ((per_op.total_tokens as f64 / unified.total_tokens as f64) - 1.0) * 100.0;

            analysis.push_str(&format!(
                "1. **Simple tasks (1 operation):** Per-Op uses {:+} tokens ({:+.1}%) vs Unified\n",
                diff, pct
            ));
            analysis.push_str("   - Higher context cost dominates\n");
            analysis.push_str("   - Tool definition overhead not amortized\n\n");
        }

        // Medium task analysis
        if self.results.len() > 1 {
            if let Some((unified, per_op)) = self.results.get(1) {
                let diff = per_op.total_tokens as i32 - unified.total_tokens as i32;
                let pct = ((per_op.total_tokens as f64 / unified.total_tokens as f64) - 1.0) * 100.0;

                analysis.push_str(&format!(
                    "2. **Medium tasks (3-5 operations):** Per-Op uses {:+} tokens ({:+.1}%) vs Unified\n",
                    diff, pct
                ));
                analysis.push_str("   - Context cost partially amortized\n");
                analysis.push_str("   - Better accuracy reduces retry cost\n\n");
            }
        }

        // Long session analysis
        if self.results.len() > 3 {
            if let Some((unified, per_op)) = self.results.get(3) {
                let diff = per_op.total_tokens as i32 - unified.total_tokens as i32;
                let pct = ((per_op.total_tokens as f64 / unified.total_tokens as f64) - 1.0) * 100.0;

                analysis.push_str(&format!(
                    "3. **Long sessions (20+ operations):** Per-Op uses {:+} tokens ({:+.1}%) vs Unified\n",
                    diff, pct
                ));
                analysis.push_str("   - Context cost fully amortized\n");
                analysis.push_str("   - Structured output saves parsing tokens\n\n");
            }
        }

        analysis
    }

    fn generate_recommendations(&self) -> String {
        let mut rec = String::new();

        let total_unified: usize = self.results.iter().map(|(u, _)| u.total_tokens).sum();
        let total_per_op: usize = self.results.iter().map(|(_, p)| p.total_tokens).sum();

        if total_unified < total_per_op {
            rec.push_str("### Unified Tools Are More Token-Efficient Overall\n\n");
            rec.push_str("**Use unified tools when:**\n");
            rec.push_str("- ✅ Running one-off commands\n");
            rec.push_str("- ✅ Short debugging sessions\n");
            rec.push_str("- ✅ Ad-hoc exploration\n");
            rec.push_str("- ✅ Token budget is tight\n\n");

            rec.push_str("**Use per-op tools when:**\n");
            rec.push_str("- ✅ Accuracy is critical (deployments, deletions)\n");
            rec.push_str("- ✅ Building TUI/GUI with rich rendering\n");
            rec.push_str("- ✅ Long monitoring sessions (20+ calls)\n");
            rec.push_str("- ✅ Parallel execution is possible\n\n");
        } else {
            rec.push_str("### Per-Operation Tools Are More Token-Efficient Overall\n\n");
            rec.push_str("**Use per-op tools when:**\n");
            rec.push_str("- ✅ Production agents\n");
            rec.push_str("- ✅ Long-running sessions\n");
            rec.push_str("- ✅ Structured output needed\n");
            rec.push_str("- ✅ Parallel execution possible\n\n");

            rec.push_str("**Use unified tools when:**\n");
            rec.push_str("- ✅ Exploratory debugging\n");
            rec.push_str("- ✅ Unsupported commands\n");
            rec.push_str("- ✅ Simple one-off tasks\n\n");
        }

        rec.push_str("### Best Practice: Hybrid Approach\n\n");
        rec.push_str("```yaml\n");
        rec.push_str("tools:\n");
        rec.push_str("  # Per-op for common operations (accurate, structured)\n");
        rec.push_str("  - docker_stats\n");
        rec.push_str("  - docker_ps\n");
        rec.push_str("  - kubectl_get\n\n");
        rec.push_str("  # Unified for flexibility (fallback)\n");
        rec.push_str("  - docker\n");
        rec.push_str("  - kubectl\n");
        rec.push_str("```\n\n");

        rec.push_str("This gives you:\n");
        rec.push_str("- ✅ Accurate per-op tools for 80% of operations\n");
        rec.push_str("- ✅ Flexible unified tools for edge cases\n");
        rec.push_str("- ✅ Best token efficiency AND accuracy\n");

        rec
    }

    /// Save report to file
    pub fn save(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let md = self.to_markdown();
        fs::write(path, md)?;
        Ok(())
    }
}
