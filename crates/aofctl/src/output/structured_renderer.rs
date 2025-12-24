//! CLI renderer for structured tool output
//!
//! Renders StructuredOutput beautifully in the terminal with:
//! - Metric cards with colored indicators
//! - ASCII tables with proper formatting
//! - Alerts with icons and colors
//! - Suggested actions with shortcuts

use aof_core::tool::{
    StructuredOutput, RenderHints, VisualizationType, MetricCard, Alert,
    AlertLevel, SuggestedAction, Threshold, TableConfig, TextAlign,
};
use super::{colors, symbols};
use std::io::{self, Write};

/// Render structured output to terminal
pub fn render_structured(output: &StructuredOutput) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // Render render_hints if available
    if let Some(hints) = &output.render_hints {
        match hints.viz_type {
            VisualizationType::Metrics => {
                render_metrics(&mut handle, &hints.metrics)?;
            }
            VisualizationType::Table => {
                if let Some(table) = &hints.table {
                    if let Some(data) = &output.data {
                        render_table(&mut handle, table, data)?;
                    }
                }
            }
            VisualizationType::Json => {
                if let Some(data) = &output.data {
                    writeln!(handle, "{}", serde_json::to_string_pretty(data).unwrap_or_default())?;
                }
            }
            _ => {
                // For other types, fall back to text
                writeln!(handle, "{}", output.text)?;
            }
        }

        // Render alerts
        if !hints.alerts.is_empty() {
            writeln!(handle)?;
            render_alerts(&mut handle, &hints.alerts)?;
        }
    } else {
        // No render hints, just show text
        writeln!(handle, "{}", output.text)?;
    }

    // Render suggested actions
    if !output.actions.is_empty() {
        writeln!(handle)?;
        render_actions(&mut handle, &output.actions)?;
    }

    Ok(())
}

/// Render metric cards
fn render_metrics<W: Write>(w: &mut W, metrics: &[MetricCard]) -> io::Result<()> {
    if metrics.is_empty() {
        return Ok(());
    }

    // Calculate column width
    let term_width = terminal_size::terminal_size()
        .map(|(terminal_size::Width(w), _)| w as usize)
        .unwrap_or(80);

    let metrics_per_row = (term_width / 20).max(1).min(metrics.len());
    let col_width = term_width / metrics_per_row;

    // Render metrics in rows
    for chunk in metrics.chunks(metrics_per_row) {
        // Top border
        for _ in 0..chunk.len() {
            write!(w, "┌{:─<width$}┐ ", "", width = col_width - 3)?;
        }
        writeln!(w)?;

        // Label row
        for metric in chunk {
            let label = truncate(&metric.label, col_width - 4);
            write!(w, "│ {}{:<width$}{}│ ",
                colors::DIM,
                label,
                colors::RESET,
                width = col_width - 4
            )?;
        }
        writeln!(w)?;

        // Value row with threshold color
        for metric in chunk {
            let color = match metric.threshold {
                Some(Threshold::Critical) => colors::RED,
                Some(Threshold::Warning) => colors::YELLOW,
                Some(Threshold::Normal) | None => colors::GREEN,
            };

            let value_str = if let Some(unit) = &metric.unit {
                format!("{} {}", metric.value, unit)
            } else {
                metric.value.clone()
            };

            let value_str = truncate(&value_str, col_width - 4);

            write!(w, "│ {}{}{:<width$}{}│ ",
                colors::BOLD,
                color,
                value_str,
                colors::RESET,
                width = col_width - 4
            )?;
        }
        writeln!(w)?;

        // Trend indicator row
        for metric in chunk {
            let trend_str = match metric.trend {
                Some(aof_core::tool::Trend::Up) => format!("{} Up", symbols::ARROW_UP),
                Some(aof_core::tool::Trend::Down) => format!("{} Down", symbols::ARROW_DOWN),
                Some(aof_core::tool::Trend::Stable) => format!("{} Stable", symbols::ARROW_RIGHT),
                None => String::new(),
            };

            let trend_str = truncate(&trend_str, col_width - 4);

            write!(w, "│ {}{:<width$}{}│ ",
                colors::DIM,
                trend_str,
                colors::RESET,
                width = col_width - 4
            )?;
        }
        writeln!(w)?;

        // Bottom border
        for _ in 0..chunk.len() {
            write!(w, "└{:─<width$}┘ ", "", width = col_width - 3)?;
        }
        writeln!(w)?;
        writeln!(w)?;
    }

    Ok(())
}

/// Render ASCII table
fn render_table<W: Write>(w: &mut W, config: &TableConfig, data: &serde_json::Value) -> io::Result<()> {
    let rows = if let Some(array) = data.as_array() {
        array
    } else if let Some(obj) = data.as_object() {
        // If data has a "rows" or similar field
        if let Some(rows) = obj.get("containers").or_else(|| obj.get("pods")).or_else(|| obj.get("data")) {
            if let Some(array) = rows.as_array() {
                array
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        }
    } else {
        return Ok(());
    };

    if rows.is_empty() {
        return Ok(());
    }

    // Calculate column widths
    let mut col_widths: Vec<usize> = config.columns.iter()
        .map(|col| col.width.unwrap_or(col.name.len().max(10)))
        .collect();

    // Auto-size columns based on data
    for (i, col) in config.columns.iter().enumerate() {
        let max_width = rows.iter()
            .filter_map(|row| row.get(&col.field))
            .filter_map(|v| Some(format_cell_value(v).len()))
            .max()
            .unwrap_or(10);

        col_widths[i] = col_widths[i].max(max_width).min(40);
    }

    // Top border
    write!(w, "┌")?;
    for (i, width) in col_widths.iter().enumerate() {
        write!(w, "{:─<width$}", "", width = width + 2)?;
        if i < col_widths.len() - 1 {
            write!(w, "┬")?;
        }
    }
    writeln!(w, "┐")?;

    // Header row
    write!(w, "│")?;
    for (col, width) in config.columns.iter().zip(&col_widths) {
        write!(w, " {}{:<width$}{} │",
            colors::BOLD,
            truncate(&col.name, *width),
            colors::RESET,
            width = width
        )?;
    }
    writeln!(w)?;

    // Header separator
    write!(w, "├")?;
    for (i, width) in col_widths.iter().enumerate() {
        write!(w, "{:─<width$}", "", width = width + 2)?;
        if i < col_widths.len() - 1 {
            write!(w, "┼")?;
        }
    }
    writeln!(w, "┤")?;

    // Data rows
    for row in rows {
        write!(w, "│")?;
        for (col, width) in config.columns.iter().zip(&col_widths) {
            let value = row.get(&col.field)
                .map(format_cell_value)
                .unwrap_or_else(|| "-".to_string());

            let value = truncate(&value, *width);

            let formatted = match col.align {
                TextAlign::Right => format!("{:>width$}", value, width = width),
                TextAlign::Center => {
                    let pad = (width - value.len()) / 2;
                    format!("{:>pad$}{:<rest$}", "", value, pad = pad, rest = width - pad)
                }
                TextAlign::Left => format!("{:<width$}", value, width = width),
            };

            write!(w, " {} │", formatted)?;
        }
        writeln!(w)?;
    }

    // Bottom border
    write!(w, "└")?;
    for (i, width) in col_widths.iter().enumerate() {
        write!(w, "{:─<width$}", "", width = width + 2)?;
        if i < col_widths.len() - 1 {
            write!(w, "┴")?;
        }
    }
    writeln!(w, "┘")?;

    Ok(())
}

/// Render alerts
fn render_alerts<W: Write>(w: &mut W, alerts: &[Alert]) -> io::Result<()> {
    for alert in alerts {
        let (icon, color) = match alert.level {
            AlertLevel::Info => (symbols::INFO, colors::BLUE),
            AlertLevel::Warning => (symbols::WARNING, colors::YELLOW),
            AlertLevel::Error => (symbols::ERROR, colors::RED),
            AlertLevel::Critical => (symbols::CRITICAL, colors::BRIGHT_RED),
        };

        write!(w, "{}{} ", color, icon)?;
        write!(w, "{}{}{}", colors::BOLD, alert.message, colors::RESET)?;

        if let Some(details) = &alert.details {
            writeln!(w)?;
            write!(w, "    {}{}{}", colors::DIM, details, colors::RESET)?;
        }

        if let Some(action) = &alert.action {
            writeln!(w)?;
            write!(w, "    {} Run: {}{}{}",
                symbols::ARROW_RIGHT,
                colors::CYAN,
                action.command,
                colors::RESET
            )?;
        }

        writeln!(w)?;
    }

    Ok(())
}

/// Render suggested actions
fn render_actions<W: Write>(w: &mut W, actions: &[SuggestedAction]) -> io::Result<()> {
    writeln!(w, "{}{}Actions:{}",
        colors::BOLD,
        colors::BRIGHT_BLUE,
        colors::RESET
    )?;

    for (i, action) in actions.iter().enumerate() {
        let confirm_mark = if action.confirm { " ⚠️" } else { "" };

        write!(w, "  {}{}. {}{}{}{} ",
            colors::CYAN,
            i + 1,
            colors::BOLD,
            action.label,
            confirm_mark,
            colors::RESET
        )?;

        if let Some(desc) = &action.description {
            write!(w, "{}{}{}", colors::DIM, desc, colors::RESET)?;
        }

        writeln!(w)?;
        writeln!(w, "     {} {}{}{}",
            symbols::ARROW_RIGHT,
            colors::GREEN,
            action.command,
            colors::RESET
        )?;
    }

    Ok(())
}

/// Format cell value for display
fn format_cell_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "-".to_string(),
        _ => serde_json::to_string(value).unwrap_or_else(|_| "-".to_string()),
    }
}

/// Truncate string to max length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        "...".to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Additional symbols
mod symbols_ext {
    use super::symbols;

    pub const ARROW_UP: &str = "↑";
    pub const ARROW_DOWN: &str = "↓";
    pub const INFO: &str = "ℹ";
    pub const WARNING: &str = "⚠";
    pub const ERROR: &str = "✗";
    pub const CRITICAL: &str = "🔥";
}

use symbols_ext as symbols;
