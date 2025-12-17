//! Beautiful CLI output formatting for AOF
//!
//! Provides professional, geeky terminal output with colors, symbols, and structure.

use std::io::{self, Write};

/// ANSI color codes for terminal styling
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const ITALIC: &str = "\x1b[3m";
    pub const UNDERLINE: &str = "\x1b[4m";

    // Foreground colors
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";
    pub const GRAY: &str = "\x1b[90m";

    // Bright colors
    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
    pub const BRIGHT_BLUE: &str = "\x1b[94m";
    pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";

    // Background colors
    pub const BG_RED: &str = "\x1b[41m";
    pub const BG_GREEN: &str = "\x1b[42m";
    pub const BG_YELLOW: &str = "\x1b[43m";
    pub const BG_BLUE: &str = "\x1b[44m";
}

/// Unicode symbols for terminal output
pub mod symbols {
    pub const CHECK: &str = "✓";
    pub const CROSS: &str = "✗";
    pub const ARROW_RIGHT: &str = "→";
    pub const ARROW_DOWN: &str = "↓";
    pub const BULLET: &str = "•";
    pub const DIAMOND: &str = "◆";
    pub const CIRCLE: &str = "●";
    pub const CIRCLE_EMPTY: &str = "○";
    pub const SQUARE: &str = "■";
    pub const TRIANGLE: &str = "▶";
    pub const STAR: &str = "★";
    pub const LIGHTNING: &str = "⚡";
    pub const GEAR: &str = "⚙";
    pub const BRAIN: &str = "🧠";
    pub const ROBOT: &str = "🤖";
    pub const ROCKET: &str = "🚀";
    pub const WARNING: &str = "⚠";
    pub const INFO: &str = "ℹ";
    pub const FLAME: &str = "🔥";
    pub const TARGET: &str = "🎯";
    pub const CLOCK: &str = "⏱";
    pub const LINK: &str = "🔗";
    pub const BOX_H: &str = "─";
    pub const BOX_V: &str = "│";
    pub const BOX_TL: &str = "┌";
    pub const BOX_TR: &str = "┐";
    pub const BOX_BL: &str = "└";
    pub const BOX_BR: &str = "┘";
    pub const BOX_T: &str = "┬";
    pub const BOX_B: &str = "┴";
    pub const BOX_L: &str = "├";
    pub const BOX_R: &str = "┤";
    pub const BOX_CROSS: &str = "┼";
    pub const SPINNER: [&str; 4] = ["◐", "◓", "◑", "◒"];
}

use colors::*;
use symbols::*;

/// Fleet output formatter for beautiful CLI display
pub struct FleetOutput {
    /// Whether to use colors (auto-detected from terminal)
    use_colors: bool,
    /// Quiet mode - minimal output
    quiet: bool,
}

impl Default for FleetOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl FleetOutput {
    pub fn new() -> Self {
        Self {
            use_colors: atty::is(atty::Stream::Stdout),
            quiet: false,
        }
    }

    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Print the AOF banner
    pub fn print_banner(&self) {
        if self.quiet {
            return;
        }
        let banner = format!(
            r#"
{CYAN}{BOLD}    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║   {BRIGHT_CYAN}█████╗  ██████╗ ███████╗{CYAN}                                    ║
    ║  {BRIGHT_CYAN}██╔══██╗██╔═══██╗██╔════╝{CYAN}   {WHITE}Agentic Ops Framework{CYAN}           ║
    ║  {BRIGHT_CYAN}███████║██║   ██║█████╗{CYAN}     {DIM}Multi-Model RCA Engine{CYAN}           ║
    ║  {BRIGHT_CYAN}██╔══██║██║   ██║██╔══╝{CYAN}                                      ║
    ║  {BRIGHT_CYAN}██║  ██║╚██████╔╝██║{CYAN}        {DIM}v0.1.x{CYAN}                          ║
    ║  {BRIGHT_CYAN}╚═╝  ╚═╝ ╚═════╝ ╚═╝{CYAN}                                         ║
    ║                                                               ║
    ╚═══════════════════════════════════════════════════════════════╝{RESET}
"#
        );
        println!("{}", banner);
    }

    /// Print fleet initialization header
    pub fn print_fleet_header(&self, fleet_name: &str, agent_count: usize, mode: &str) {
        if self.quiet {
            return;
        }
        println!();
        self.print_section_header(&format!("Fleet: {}", fleet_name));
        println!(
            "  {GRAY}{GEAR}{RESET} Mode: {CYAN}{}{RESET}  {GRAY}│{RESET}  Agents: {CYAN}{}{RESET}",
            mode, agent_count
        );
        println!();
    }

    /// Print a section header with decorative line
    pub fn print_section_header(&self, title: &str) {
        let width = 60;
        let title_len = title.len() + 4;
        let padding = if width > title_len {
            (width - title_len) / 2
        } else {
            0
        };

        println!(
            "{CYAN}{}{BOLD} {} {RESET}{CYAN}{}{RESET}",
            BOX_H.repeat(padding),
            title,
            BOX_H.repeat(width - padding - title_len)
        );
    }

    /// Print tier execution start
    pub fn print_tier_start(&self, tier: u32, agent_names: &[String], consensus: &str) {
        if self.quiet {
            return;
        }
        let tier_color = match tier {
            1 => BRIGHT_BLUE,
            2 => BRIGHT_MAGENTA,
            3 => BRIGHT_GREEN,
            _ => CYAN,
        };

        println!(
            "\n{tier_color}{BOLD}╭─────────────────────────────────────────────────────────────╮{RESET}"
        );
        println!(
            "{tier_color}{BOLD}│{RESET} {LIGHTNING} {tier_color}TIER {}{RESET}  {GRAY}│{RESET}  Consensus: {YELLOW}{}{RESET}",
            tier, consensus
        );
        println!(
            "{tier_color}{BOLD}╰─────────────────────────────────────────────────────────────╯{RESET}"
        );

        for name in agent_names {
            println!(
                "  {tier_color}{TRIANGLE}{RESET} {WHITE}{}{RESET}",
                name
            );
        }
    }

    /// Print tier completion
    pub fn print_tier_complete(&self, tier: u32, results: usize, confidence: f64, duration_ms: u64) {
        if self.quiet {
            return;
        }
        let confidence_color = if confidence >= 0.8 {
            BRIGHT_GREEN
        } else if confidence >= 0.5 {
            YELLOW
        } else {
            RED
        };

        let confidence_bar = self.confidence_bar(confidence);

        println!(
            "  {GREEN}{CHECK}{RESET} Tier {} complete: {CYAN}{}{RESET} results  {GRAY}│{RESET}  {confidence_color}{:.0}%{RESET} {confidence_bar}  {GRAY}│{RESET}  {DIM}{}ms{RESET}",
            tier, results, confidence * 100.0, duration_ms
        );
    }

    /// Generate a visual confidence bar
    fn confidence_bar(&self, confidence: f64) -> String {
        let filled = (confidence * 10.0) as usize;
        let empty = 10 - filled;

        let bar_color = if confidence >= 0.8 {
            BRIGHT_GREEN
        } else if confidence >= 0.5 {
            YELLOW
        } else {
            RED
        };

        format!(
            "{bar_color}{}{}{}",
            "█".repeat(filled),
            DIM,
            "░".repeat(empty)
        )
    }

    /// Print agent execution
    pub fn print_agent_executing(&self, agent_name: &str, model: &str) {
        if self.quiet {
            return;
        }
        println!(
            "    {GRAY}{CIRCLE}{RESET} {WHITE}{}{RESET} {DIM}({model}){RESET}",
            agent_name
        );
    }

    /// Print agent completion
    pub fn print_agent_complete(&self, agent_name: &str, duration_ms: u64) {
        if self.quiet {
            return;
        }
        println!(
            "    {GREEN}{CHECK}{RESET} {WHITE}{}{RESET} {DIM}{}ms{RESET}",
            agent_name, duration_ms
        );
    }

    /// Print the final RCA report beautifully
    pub fn print_rca_report(&self, report: &serde_json::Value) {
        println!();
        println!();

        // Extract report content
        let result = report
            .get("result")
            .and_then(|r| r.as_str())
            .unwrap_or("");

        let tier_count = report
            .get("tier_count")
            .and_then(|t| t.as_u64())
            .unwrap_or(0);

        let synthesized_by = report
            .get("synthesized_by")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");

        // Print beautiful report header
        println!(
            "{BRIGHT_GREEN}{BOLD}╔═══════════════════════════════════════════════════════════════════════════╗{RESET}"
        );
        println!(
            "{BRIGHT_GREEN}{BOLD}║{RESET}  {TARGET} {WHITE}{BOLD}ROOT CAUSE ANALYSIS REPORT{RESET}                                          {BRIGHT_GREEN}{BOLD}║{RESET}"
        );
        println!(
            "{BRIGHT_GREEN}{BOLD}╠═══════════════════════════════════════════════════════════════════════════╣{RESET}"
        );
        println!(
            "{BRIGHT_GREEN}{BOLD}║{RESET}  {GRAY}Tiers: {CYAN}{}{RESET}  {GRAY}│{RESET}  {GRAY}Synthesized by: {CYAN}{}{RESET}                          {BRIGHT_GREEN}{BOLD}║{RESET}",
            tier_count, synthesized_by
        );
        println!(
            "{BRIGHT_GREEN}{BOLD}╚═══════════════════════════════════════════════════════════════════════════╝{RESET}"
        );
        println!();

        // Parse and pretty-print the markdown content
        self.print_markdown_report(result);

        println!();
        println!(
            "{DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{RESET}"
        );
        println!(
            "{DIM}  Generated by AOF Multi-Model Consensus Engine{RESET}"
        );
    }

    /// Parse and beautifully print markdown content
    fn print_markdown_report(&self, markdown: &str) {
        for line in markdown.lines() {
            if line.starts_with("# ") {
                // H1 header
                let title = &line[2..];
                println!();
                println!(
                    "{BRIGHT_CYAN}{BOLD}{}{RESET}",
                    title
                );
                println!(
                    "{CYAN}{}",
                    "═".repeat(title.len().min(75))
                );
            } else if line.starts_with("## ") {
                // H2 header
                let title = &line[3..];
                println!();
                println!(
                    "{BRIGHT_YELLOW}{BOLD}{} {}{RESET}",
                    DIAMOND, title
                );
            } else if line.starts_with("### ") {
                // H3 header
                let title = &line[4..];
                println!(
                    "  {CYAN}{} {}{RESET}",
                    TRIANGLE, title
                );
            } else if line.starts_with("- [ ] ") {
                // Unchecked checkbox
                let item = &line[6..];
                println!(
                    "  {YELLOW}{CIRCLE_EMPTY}{RESET} {WHITE}{}{RESET}",
                    item
                );
            } else if line.starts_with("- [x] ") || line.starts_with("- [X] ") {
                // Checked checkbox
                let item = &line[6..];
                println!(
                    "  {GREEN}{CHECK}{RESET} {DIM}{}{RESET}",
                    item
                );
            } else if line.starts_with("- ") || line.starts_with("* ") {
                // Bullet point
                let item = &line[2..];
                let formatted = self.format_inline_markdown(item);
                println!(
                    "  {GRAY}{BULLET}{RESET} {}",
                    formatted
                );
            } else if line.starts_with("1. ") || line.starts_with("2. ") || line.starts_with("3. ") || line.starts_with("4. ") || line.starts_with("5. ") {
                // Numbered list
                let formatted = self.format_inline_markdown(line);
                println!("  {}", formatted);
            } else if line.starts_with("|") && line.contains("|") {
                // Table row
                self.print_table_row(line);
            } else if line.starts_with("```") {
                // Code block delimiter
                println!("{DIM}  ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄{RESET}");
            } else if line.starts_with("**Category**:") || line.starts_with("**Description**:") || line.starts_with("**Confidence**:") {
                // Key-value pairs
                let formatted = self.format_inline_markdown(line);
                println!("  {}", formatted);
            } else if line.starts_with("---") {
                // Horizontal rule
                println!(
                    "{DIM}  ────────────────────────────────────────────────────────{RESET}"
                );
            } else if line.trim().is_empty() {
                // Empty line
                println!();
            } else {
                // Regular text
                let formatted = self.format_inline_markdown(line);
                println!("  {}", formatted);
            }
        }
    }

    /// Format inline markdown (bold, code, etc.)
    fn format_inline_markdown(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Bold text **text**
        while let Some(start) = result.find("**") {
            if let Some(end) = result[start + 2..].find("**") {
                let before = &result[..start];
                let bold_text = &result[start + 2..start + 2 + end];
                let after = &result[start + 2 + end + 2..];
                result = format!("{}{BOLD}{WHITE}{}{RESET}{}", before, bold_text, after);
            } else {
                break;
            }
        }

        // Inline code `text`
        while let Some(start) = result.find('`') {
            if let Some(end) = result[start + 1..].find('`') {
                let before = &result[..start];
                let code_text = &result[start + 1..start + 1 + end];
                let after = &result[start + 1 + end + 1..];
                result = format!("{}{CYAN}{}{RESET}{}", before, code_text, after);
            } else {
                break;
            }
        }

        result
    }

    /// Print a markdown table row beautifully
    fn print_table_row(&self, line: &str) {
        let cells: Vec<&str> = line.split('|').filter(|s| !s.trim().is_empty()).collect();

        if cells.is_empty() {
            return;
        }

        // Check if it's a separator row (e.g., |---|---|)
        if cells.iter().all(|c| c.trim().chars().all(|ch| ch == '-' || ch == ':')) {
            println!(
                "  {GRAY}├{}┤{RESET}",
                "─".repeat(cells.len() * 15 + cells.len() - 1)
            );
            return;
        }

        // Print cells
        print!("  {GRAY}│{RESET}");
        for cell in cells {
            let cell_text = cell.trim();
            // Color checkmarks and X marks
            let formatted = if cell_text == "✓" {
                format!("{GREEN}{CHECK}{RESET}")
            } else if cell_text == "✗" {
                format!("{RED}{CROSS}{RESET}")
            } else if cell_text == "HIGH" {
                format!("{BRIGHT_GREEN}{BOLD}HIGH{RESET}")
            } else if cell_text == "MEDIUM" {
                format!("{YELLOW}MEDIUM{RESET}")
            } else if cell_text == "LOW" {
                format!("{RED}LOW{RESET}")
            } else {
                format!("{WHITE}{}{RESET}", cell_text)
            };
            print!(" {:^13} {GRAY}│{RESET}", formatted);
        }
        println!();
    }

    /// Print task submission
    pub fn print_task_submitted(&self, task_id: &str) {
        if self.quiet {
            return;
        }
        println!(
            "\n{BLUE}{INFO}{RESET} Task submitted: {CYAN}{}{RESET}",
            &task_id[..8]
        );
    }

    /// Print fleet completion summary
    pub fn print_fleet_complete(&self, fleet_name: &str, duration_ms: u64, cost_estimate: Option<f64>) {
        if self.quiet {
            return;
        }
        println!();
        println!(
            "{GREEN}{BOLD}╭─────────────────────────────────────────────────────────────╮{RESET}"
        );
        println!(
            "{GREEN}{BOLD}│{RESET} {ROCKET} {WHITE}{BOLD}FLEET EXECUTION COMPLETE{RESET}                                  {GREEN}{BOLD}│{RESET}"
        );
        println!(
            "{GREEN}{BOLD}├─────────────────────────────────────────────────────────────┤{RESET}"
        );
        println!(
            "{GREEN}{BOLD}│{RESET}  Fleet: {CYAN}{:<25}{RESET}                         {GREEN}{BOLD}│{RESET}",
            fleet_name
        );
        println!(
            "{GREEN}{BOLD}│{RESET}  Duration: {YELLOW}{:.2}s{RESET}                                            {GREEN}{BOLD}│{RESET}",
            duration_ms as f64 / 1000.0
        );
        if let Some(cost) = cost_estimate {
            println!(
                "{GREEN}{BOLD}│{RESET}  Est. Cost: {MAGENTA}${:.4}{RESET}                                          {GREEN}{BOLD}│{RESET}",
                cost
            );
        }
        println!(
            "{GREEN}{BOLD}╰─────────────────────────────────────────────────────────────╯{RESET}"
        );
    }

    /// Print error message
    pub fn print_error(&self, message: &str) {
        eprintln!(
            "{RED}{BOLD}{CROSS} Error:{RESET} {WHITE}{}{RESET}",
            message
        );
    }

    /// Print warning message
    pub fn print_warning(&self, message: &str) {
        eprintln!(
            "{YELLOW}{WARNING} Warning:{RESET} {WHITE}{}{RESET}",
            message
        );
    }

    /// Print info message
    pub fn print_info(&self, message: &str) {
        println!(
            "{BLUE}{INFO}{RESET} {WHITE}{}{RESET}",
            message
        );
    }

    /// Print success message
    pub fn print_success(&self, message: &str) {
        println!(
            "{GREEN}{CHECK}{RESET} {WHITE}{}{RESET}",
            message
        );
    }

    /// Flush stdout
    pub fn flush(&self) {
        let _ = io::stdout().flush();
    }
}

/// Simple spinner for progress indication
pub struct Spinner {
    state: usize,
    message: String,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        Self {
            state: 0,
            message: message.to_string(),
        }
    }

    pub fn tick(&mut self) {
        self.state = (self.state + 1) % symbols::SPINNER.len();
        print!(
            "\r{CYAN}{}{RESET} {}",
            symbols::SPINNER[self.state],
            self.message
        );
        let _ = io::stdout().flush();
    }

    pub fn finish(&self, success: bool) {
        if success {
            println!(
                "\r{GREEN}{CHECK}{RESET} {}",
                self.message
            );
        } else {
            println!(
                "\r{RED}{CROSS}{RESET} {}",
                self.message
            );
        }
    }
}
