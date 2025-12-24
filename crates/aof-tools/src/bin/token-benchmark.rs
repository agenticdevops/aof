//! Token efficiency benchmark CLI tool
//!
//! Run comprehensive benchmarks comparing unified vs per-operation tools

#[cfg(feature = "benchmark")]
fn main() {
    use aof_tools::benchmark::run_full_benchmark;

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║        AOF Token Efficiency Benchmark                        ║");
    println!("║        Unified vs Per-Operation Tools                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let report = run_full_benchmark();

    println!("\n\n═══════════════════════════════════════════════════════════════");
    println!("                     FINAL RESULTS");
    println!("═══════════════════════════════════════════════════════════════\n");

    // Calculate totals
    let total_unified: usize = report.results.iter().map(|(u, _)| u.total_tokens).sum();
    let total_per_op: usize = report.results.iter().map(|(_, p)| p.total_tokens).sum();
    let diff_pct = ((total_per_op as f64 / total_unified as f64) - 1.0) * 100.0;

    println!("TOTAL TOKENS (all scenarios):");
    println!("  Unified:  {} tokens", total_unified);
    println!("  Per-Op:   {} tokens", total_per_op);
    println!("  Diff:     {:+} tokens ({:+.1}%)\n", total_per_op as i32 - total_unified as i32, diff_pct);

    let winner = if total_unified < total_per_op {
        "✅ UNIFIED TOOLS ARE MORE TOKEN-EFFICIENT"
    } else {
        "✅ PER-OPERATION TOOLS ARE MORE TOKEN-EFFICIENT"
    };

    println!("{}\n", winner);

    // Save report
    let report_path = "docs/internal/TOKEN-BENCHMARK-REPORT.md";
    match report.save(report_path) {
        Ok(_) => println!("📝 Full report saved to: {}", report_path),
        Err(e) => eprintln!("❌ Failed to save report: {}", e),
    }

    println!("\n═══════════════════════════════════════════════════════════════\n");
}

#[cfg(not(feature = "benchmark"))]
fn main() {
    eprintln!("Benchmark feature not enabled!");
    eprintln!("Run with: cargo run --bin token-benchmark --features benchmark");
    std::process::exit(1);
}
