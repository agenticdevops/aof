//! Structured tool output for TUI/GUI rendering
//!
//! This module provides rich, semantic output formats that can be rendered
//! beautifully in CLI, TUI (ratatui), and GUI (web/desktop) contexts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Structured tool output with rendering hints
///
/// # Examples
///
/// ```
/// use aof_core::tool::StructuredOutput;
///
/// let output = StructuredOutput::metrics()
///     .add_metric("Running Containers", "9", None)
///     .add_metric("Total CPU", "47.5", Some("%"))
///     .with_text("Retrieved 9 running containers");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredOutput {
    /// Plain text output (backward compatibility)
    pub text: String,

    /// Structured data (JSON-serializable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    /// Rendering hints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render_hints: Option<RenderHints>,

    /// Suggested actions
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub actions: Vec<SuggestedAction>,

    /// Output status
    #[serde(default)]
    pub status: OutputStatus,
}

impl StructuredOutput {
    /// Create plain text output
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            data: None,
            render_hints: None,
            actions: Vec::new(),
            status: OutputStatus::Success,
        }
    }

    /// Create metrics dashboard output
    pub fn metrics() -> StructuredOutputBuilder {
        StructuredOutputBuilder::new(VisualizationType::Metrics)
    }

    /// Create table output
    pub fn table() -> StructuredOutputBuilder {
        StructuredOutputBuilder::new(VisualizationType::Table)
    }

    /// Create chart output
    pub fn chart(chart_type: ChartType) -> StructuredOutputBuilder {
        StructuredOutputBuilder::new(VisualizationType::Chart)
            .with_chart_type(chart_type)
    }

    /// Create JSON output
    pub fn json(data: serde_json::Value) -> Self {
        Self {
            text: serde_json::to_string_pretty(&data).unwrap_or_default(),
            data: Some(data),
            render_hints: Some(RenderHints {
                viz_type: VisualizationType::Json,
                table: None,
                chart: None,
                metrics: Vec::new(),
                alerts: Vec::new(),
            }),
            actions: Vec::new(),
            status: OutputStatus::Success,
        }
    }

    /// Add suggested action
    pub fn with_action(mut self, action: SuggestedAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Set status
    pub fn with_status(mut self, status: OutputStatus) -> Self {
        self.status = status;
        self
    }

    /// Set text
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }
}

/// Builder for structured output
pub struct StructuredOutputBuilder {
    viz_type: VisualizationType,
    text: String,
    data: Option<serde_json::Value>,
    table: Option<TableConfig>,
    chart: Option<ChartConfig>,
    metrics: Vec<MetricCard>,
    alerts: Vec<Alert>,
    actions: Vec<SuggestedAction>,
    status: OutputStatus,
}

impl StructuredOutputBuilder {
    pub fn new(viz_type: VisualizationType) -> Self {
        Self {
            viz_type,
            text: String::new(),
            data: None,
            table: None,
            chart: None,
            metrics: Vec::new(),
            alerts: Vec::new(),
            actions: Vec::new(),
            status: OutputStatus::Success,
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_table(mut self, table: TableConfig) -> Self {
        self.table = Some(table);
        self
    }

    pub fn with_chart_type(mut self, chart_type: ChartType) -> Self {
        if self.chart.is_none() {
            self.chart = Some(ChartConfig {
                chart_type,
                x_axis: String::new(),
                y_axis: Vec::new(),
                time_series: false,
            });
        } else if let Some(chart) = &mut self.chart {
            chart.chart_type = chart_type;
        }
        self
    }

    pub fn add_metric(mut self, label: impl Into<String>, value: impl Into<String>, unit: Option<impl Into<String>>) -> Self {
        self.metrics.push(MetricCard {
            label: label.into(),
            value: value.into(),
            unit: unit.map(|u| u.into()),
            trend: None,
            threshold: None,
            sparkline: None,
        });
        self
    }

    pub fn add_metric_with_threshold(
        mut self,
        label: impl Into<String>,
        value: impl Into<String>,
        unit: Option<impl Into<String>>,
        threshold: Threshold,
    ) -> Self {
        self.metrics.push(MetricCard {
            label: label.into(),
            value: value.into(),
            unit: unit.map(|u| u.into()),
            trend: None,
            threshold: Some(threshold),
            sparkline: None,
        });
        self
    }

    pub fn add_alert(mut self, alert: Alert) -> Self {
        self.alerts.push(alert);
        self
    }

    pub fn add_action(mut self, action: SuggestedAction) -> Self {
        self.actions.push(action);
        self
    }

    pub fn with_status(mut self, status: OutputStatus) -> Self {
        self.status = status;
        self
    }

    pub fn build(self) -> StructuredOutput {
        StructuredOutput {
            text: self.text,
            data: self.data,
            render_hints: Some(RenderHints {
                viz_type: self.viz_type,
                table: self.table,
                chart: self.chart,
                metrics: self.metrics,
                alerts: self.alerts,
            }),
            actions: self.actions,
            status: self.status,
        }
    }
}

/// Rendering hints for different contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderHints {
    /// Preferred visualization type
    pub viz_type: VisualizationType,

    /// Table configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table: Option<TableConfig>,

    /// Chart configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart: Option<ChartConfig>,

    /// Metrics for dashboard cards
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub metrics: Vec<MetricCard>,

    /// Alerts/warnings
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub alerts: Vec<Alert>,
}

/// Visualization type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VisualizationType {
    /// Tabular data
    Table,
    /// Time series, bar chart, pie chart
    Chart,
    /// Dashboard-style metric cards
    Metrics,
    /// Hierarchical data
    Tree,
    /// Raw JSON viewer
    Json,
    /// Log stream viewer
    Logs,
    /// Side-by-side diff
    Diff,
    /// Event timeline
    Timeline,
    /// Network graph
    Network,
}

/// Table configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
    /// Column definitions
    pub columns: Vec<TableColumn>,

    /// Enable sorting
    #[serde(default)]
    pub sortable: bool,

    /// Enable filtering
    #[serde(default)]
    pub filterable: bool,

    /// Row color conditions (condition -> color)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_colors: Option<HashMap<String, String>>,
}

/// Table column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    /// Column name
    pub name: String,

    /// Field name in data
    pub field: String,

    /// Column width (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<usize>,

    /// Alignment
    #[serde(default)]
    pub align: TextAlign,

    /// Format hint (e.g., "bytes", "duration", "percentage")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Text alignment
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    #[default]
    Left,
    Right,
    Center,
}

/// Chart configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// Chart type
    pub chart_type: ChartType,

    /// X-axis field
    pub x_axis: String,

    /// Y-axis fields
    pub y_axis: Vec<String>,

    /// Is this a time series
    #[serde(default)]
    pub time_series: bool,
}

/// Chart type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    Line,
    Bar,
    Pie,
    Area,
    Scatter,
}

/// Metric card for dashboards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricCard {
    /// Metric label
    pub label: String,

    /// Current value
    pub value: String,

    /// Unit (e.g., "%", "GB", "ms")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    /// Trend indicator
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trend: Option<Trend>,

    /// Threshold status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<Threshold>,

    /// Sparkline data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sparkline: Option<Vec<f64>>,
}

/// Trend direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Trend {
    Up,
    Down,
    Stable,
}

/// Threshold status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Threshold {
    Normal,
    Warning,
    Critical,
}

/// Alert message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert level
    pub level: AlertLevel,

    /// Alert message
    pub message: String,

    /// Additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,

    /// Suggested action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<SuggestedAction>,
}

/// Alert level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Suggested action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    /// Action label
    pub label: String,

    /// Command to execute
    pub command: String,

    /// Action description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Requires confirmation
    #[serde(default)]
    pub confirm: bool,
}

/// Output status
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputStatus {
    #[default]
    Success,
    Warning,
    Error,
}
