//! Workflow Visualization UI
//!
//! Provides an Argo Workflows-style terminal UI for visualizing
//! workflow execution with DAG graph representation.

use aof_core::{StepStatus, StepType, Workflow, WorkflowState, WorkflowStatus};
use aof_runtime::WorkflowEvent;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use std::collections::HashMap;
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Node in the workflow graph
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub name: String,
    pub step_type: StepType,
    pub status: NodeStatus,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
}

/// Status of a graph node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    WaitingApproval,
}

impl From<StepStatus> for NodeStatus {
    fn from(status: StepStatus) -> Self {
        match status {
            StepStatus::Pending => NodeStatus::Pending,
            StepStatus::Running => NodeStatus::Running,
            StepStatus::Completed => NodeStatus::Completed,
            StepStatus::Failed => NodeStatus::Failed,
            StepStatus::Skipped => NodeStatus::Skipped,
        }
    }
}

/// Workflow visualization state
pub struct WorkflowVisualization {
    pub workflow: Workflow,
    pub nodes: HashMap<String, GraphNode>,
    pub execution_order: Vec<String>,
    pub current_step: Option<String>,
    pub status: WorkflowStatus,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
    pub logs: Vec<String>,
    pub selected_node: Option<String>,
    pub scroll_offset: u16,
}

impl WorkflowVisualization {
    pub fn new(workflow: Workflow) -> Self {
        let mut nodes = HashMap::new();
        let mut execution_order = Vec::new();

        // Build nodes from workflow steps
        for step in &workflow.spec.steps {
            nodes.insert(
                step.name.clone(),
                GraphNode {
                    name: step.name.clone(),
                    step_type: step.step_type,
                    status: NodeStatus::Pending,
                    duration_ms: None,
                    error: None,
                },
            );
            execution_order.push(step.name.clone());
        }

        Self {
            workflow,
            nodes,
            execution_order,
            current_step: None,
            status: WorkflowStatus::Pending,
            start_time: None,
            end_time: None,
            logs: Vec::new(),
            selected_node: None,
            scroll_offset: 0,
        }
    }

    /// Update visualization from workflow event
    pub fn handle_event(&mut self, event: WorkflowEvent) {
        match event {
            WorkflowEvent::Started { workflow_name, .. } => {
                self.start_time = Some(Instant::now());
                self.status = WorkflowStatus::Running;
                self.logs
                    .push(format!("[START] Workflow '{}' started", workflow_name));
            }
            WorkflowEvent::StepStarted { step_name } => {
                self.current_step = Some(step_name.clone());
                if let Some(node) = self.nodes.get_mut(&step_name) {
                    node.status = NodeStatus::Running;
                }
                self.logs.push(format!("[STEP] Starting: {}", step_name));
            }
            WorkflowEvent::StepCompleted {
                step_name,
                duration_ms,
            } => {
                if let Some(node) = self.nodes.get_mut(&step_name) {
                    node.status = NodeStatus::Completed;
                    node.duration_ms = Some(duration_ms);
                }
                self.logs
                    .push(format!("[DONE] {} ({}ms)", step_name, duration_ms));
            }
            WorkflowEvent::StepFailed { step_name, error } => {
                if let Some(node) = self.nodes.get_mut(&step_name) {
                    node.status = NodeStatus::Failed;
                    node.error = Some(error.clone());
                }
                self.logs
                    .push(format!("[FAIL] {}: {}", step_name, error));
            }
            WorkflowEvent::WaitingApproval { step_name, .. } => {
                if let Some(node) = self.nodes.get_mut(&step_name) {
                    node.status = NodeStatus::WaitingApproval;
                }
                self.logs
                    .push(format!("[WAIT] Approval needed: {}", step_name));
            }
            WorkflowEvent::Completed { status, .. } => {
                self.status = status;
                self.end_time = Some(Instant::now());
                self.current_step = None;
                self.logs.push(format!("[END] Status: {:?}", status));
            }
            WorkflowEvent::Error { message } => {
                self.logs.push(format!("[ERROR] {}", message));
            }
            _ => {}
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        if let Some(start) = self.start_time {
            if let Some(end) = self.end_time {
                end.duration_since(start)
            } else {
                start.elapsed()
            }
        } else {
            Duration::ZERO
        }
    }

    /// Count completed steps
    pub fn completed_count(&self) -> usize {
        self.nodes
            .values()
            .filter(|n| n.status == NodeStatus::Completed)
            .count()
    }

    /// Get progress percentage
    pub fn progress(&self) -> f64 {
        if self.nodes.is_empty() {
            return 0.0;
        }
        (self.completed_count() as f64 / self.nodes.len() as f64) * 100.0
    }
}

/// Run the workflow visualization UI
pub async fn run_visualization(
    workflow: Workflow,
    mut event_rx: mpsc::Receiver<WorkflowEvent>,
) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize visualization
    let mut viz = WorkflowVisualization::new(workflow);

    // Main loop
    loop {
        // Handle incoming workflow events (non-blocking)
        while let Ok(event) = event_rx.try_recv() {
            let is_completed = matches!(event, WorkflowEvent::Completed { .. });
            viz.handle_event(event);
            if is_completed {
                // Keep showing for a moment after completion
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        // Handle user input (non-blocking)
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c')
                        if key.modifiers == crossterm::event::KeyModifiers::CONTROL =>
                    {
                        break
                    }
                    KeyCode::Up => {
                        viz.scroll_offset = viz.scroll_offset.saturating_add(1);
                    }
                    KeyCode::Down => {
                        viz.scroll_offset = viz.scroll_offset.saturating_sub(1);
                    }
                    KeyCode::Tab => {
                        // Cycle through nodes
                        let current_idx = viz
                            .selected_node
                            .as_ref()
                            .and_then(|n| viz.execution_order.iter().position(|x| x == n))
                            .unwrap_or(0);
                        let next_idx = (current_idx + 1) % viz.execution_order.len();
                        viz.selected_node = Some(viz.execution_order[next_idx].clone());
                    }
                    _ => {}
                }
            }
        }

        // Draw UI
        terminal.draw(|f| render_workflow_ui(f, &viz))?;

        // Check if workflow completed
        if matches!(
            viz.status,
            WorkflowStatus::Completed | WorkflowStatus::Failed | WorkflowStatus::Cancelled
        ) {
            // Wait for user to dismiss
            loop {
                if crossterm::event::poll(Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        if matches!(
                            key.code,
                            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter
                        ) {
                            break;
                        }
                    }
                }
                terminal.draw(|f| render_workflow_ui(f, &viz))?;
            }
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// Render the workflow visualization UI
fn render_workflow_ui(f: &mut Frame, viz: &WorkflowVisualization) {
    // Main layout: header, content, footer
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Content
            Constraint::Length(3),  // Footer
        ])
        .split(f.size());

    // Header
    render_header(f, viz, main_layout[0]);

    // Content: graph on left, details on right
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_layout[1]);

    // Graph view
    render_graph(f, viz, content_layout[0]);

    // Details panel (split into node details and logs)
    let details_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(content_layout[1]);

    render_node_details(f, viz, details_layout[0]);
    render_logs(f, viz, details_layout[1]);

    // Footer
    render_footer(f, viz, main_layout[2]);
}

fn render_header(f: &mut Frame, viz: &WorkflowVisualization, area: Rect) {
    let status_color = match viz.status {
        WorkflowStatus::Pending => Color::Gray,
        WorkflowStatus::Running => Color::Yellow,
        WorkflowStatus::WaitingApproval | WorkflowStatus::WaitingInput => Color::Cyan,
        WorkflowStatus::Completed => Color::Green,
        WorkflowStatus::Failed => Color::Red,
        WorkflowStatus::Cancelled => Color::DarkGray,
    };

    let elapsed = viz.elapsed();
    let elapsed_str = format!("{:.1}s", elapsed.as_secs_f64());

    let header_text = format!(
        " {} │ Status: {:?} │ Progress: {}/{} ({:.0}%) │ Elapsed: {} ",
        viz.workflow.metadata.name.to_uppercase(),
        viz.status,
        viz.completed_count(),
        viz.nodes.len(),
        viz.progress(),
        elapsed_str
    );

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(status_color)),
        );

    f.render_widget(header, area);
}

fn render_graph(f: &mut Frame, viz: &WorkflowVisualization, area: Rect) {
    let block = Block::default()
        .title(" Workflow DAG ")
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::White));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Build the DAG visualization
    let mut lines: Vec<Line> = Vec::new();

    // ASCII art style DAG
    for (i, step_name) in viz.execution_order.iter().enumerate() {
        if let Some(node) = viz.nodes.get(step_name) {
            let (icon, color) = match node.status {
                NodeStatus::Pending => ("○", Color::DarkGray),
                NodeStatus::Running => ("◉", Color::Yellow),
                NodeStatus::Completed => ("●", Color::Green),
                NodeStatus::Failed => ("✗", Color::Red),
                NodeStatus::Skipped => ("◌", Color::Gray),
                NodeStatus::WaitingApproval => ("⏸", Color::Cyan),
            };

            let type_icon = match node.step_type {
                StepType::Agent => "🤖",
                StepType::Approval => "👤",
                StepType::Validation => "✓",
                StepType::Parallel => "⫘",
                StepType::Join => "⫙",
                StepType::Terminal => "◼",
            };

            let duration_str = node
                .duration_ms
                .map(|d| format!(" ({}ms)", d))
                .unwrap_or_default();

            let is_selected = viz.selected_node.as_ref() == Some(step_name);
            let style = if is_selected {
                Style::default().fg(color).add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(color)
            };

            // Node line
            let node_line = Line::from(vec![
                Span::raw("  "),
                Span::styled(icon, style),
                Span::raw(" "),
                Span::styled(type_icon, Style::default()),
                Span::raw(" "),
                Span::styled(&node.name, style),
                Span::styled(duration_str, Style::default().fg(Color::DarkGray)),
            ]);
            lines.push(node_line);

            // Connector to next step
            if i < viz.execution_order.len() - 1 {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled("│", Style::default().fg(Color::DarkGray)),
                ]));
            }
        }
    }

    let graph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((viz.scroll_offset, 0));

    f.render_widget(graph, inner);
}

fn render_node_details(f: &mut Frame, viz: &WorkflowVisualization, area: Rect) {
    let block = Block::default()
        .title(" Step Details ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::White));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let selected_name = viz
        .selected_node
        .as_ref()
        .or(viz.current_step.as_ref());

    if let Some(name) = selected_name {
        if let Some(node) = viz.nodes.get(name) {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&node.name),
                ]),
                Line::from(vec![
                    Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{:?}", node.step_type)),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        format!("{:?}", node.status),
                        Style::default().fg(match node.status {
                            NodeStatus::Completed => Color::Green,
                            NodeStatus::Failed => Color::Red,
                            NodeStatus::Running => Color::Yellow,
                            _ => Color::White,
                        }),
                    ),
                ]),
            ];

            if let Some(duration) = node.duration_ms {
                lines.push(Line::from(vec![
                    Span::styled("Duration: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{}ms", duration)),
                ]));
            }

            if let Some(ref error) = node.error {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Error: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(error, Style::default().fg(Color::Red)),
                ]));
            }

            let details = Paragraph::new(lines).wrap(Wrap { trim: true });
            f.render_widget(details, inner);
        }
    } else {
        let hint = Paragraph::new("Press TAB to select a step")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(hint, inner);
    }
}

fn render_logs(f: &mut Frame, viz: &WorkflowVisualization, area: Rect) {
    let block = Block::default()
        .title(" Event Log ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::White));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let log_items: Vec<ListItem> = viz
        .logs
        .iter()
        .rev()
        .take(inner.height as usize)
        .map(|log| {
            let style = if log.contains("[ERROR]") || log.contains("[FAIL]") {
                Style::default().fg(Color::Red)
            } else if log.contains("[DONE]") {
                Style::default().fg(Color::Green)
            } else if log.contains("[WAIT]") {
                Style::default().fg(Color::Cyan)
            } else if log.contains("[STEP]") {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            ListItem::new(log.as_str()).style(style)
        })
        .collect();

    let logs_list = List::new(log_items);
    f.render_widget(logs_list, inner);
}

fn render_footer(f: &mut Frame, viz: &WorkflowVisualization, area: Rect) {
    let footer_text = if matches!(
        viz.status,
        WorkflowStatus::Completed | WorkflowStatus::Failed | WorkflowStatus::Cancelled
    ) {
        " Press Q or Enter to exit "
    } else {
        " ↑/↓ Scroll │ TAB Select step │ Q Quit "
    };

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    f.render_widget(footer, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use aof_core::WorkflowMetadata;

    fn create_test_workflow() -> Workflow {
        Workflow {
            api_version: "aof.dev/v1".to_string(),
            kind: "Workflow".to_string(),
            metadata: WorkflowMetadata {
                name: "test-workflow".to_string(),
                namespace: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
            },
            spec: aof_core::WorkflowSpec {
                state: None,
                entrypoint: "start".to_string(),
                steps: vec![
                    aof_core::WorkflowStep {
                        name: "start".to_string(),
                        step_type: StepType::Agent,
                        agent: Some("test-agent".to_string()),
                        config: None,
                        validation: vec![],
                        next: Some(aof_core::NextStep::Simple("end".to_string())),
                        parallel: false,
                        branches: None,
                        join: None,
                        on_error: None,
                        interrupt: None,
                        status: None,
                        timeout: None,
                    },
                    aof_core::WorkflowStep {
                        name: "end".to_string(),
                        step_type: StepType::Terminal,
                        agent: None,
                        config: None,
                        validation: vec![],
                        next: None,
                        parallel: false,
                        branches: None,
                        join: None,
                        on_error: None,
                        interrupt: None,
                        status: Some(aof_core::TerminalStatus::Completed),
                        timeout: None,
                    },
                ],
                reducers: HashMap::new(),
                error_handler: None,
                retry: None,
                checkpointing: None,
                recovery: None,
                fleet: None,
            },
        }
    }

    #[test]
    fn test_visualization_creation() {
        let workflow = create_test_workflow();
        let viz = WorkflowVisualization::new(workflow);

        assert_eq!(viz.nodes.len(), 2);
        assert!(viz.nodes.contains_key("start"));
        assert!(viz.nodes.contains_key("end"));
        assert_eq!(viz.status, WorkflowStatus::Pending);
    }

    #[test]
    fn test_event_handling() {
        let workflow = create_test_workflow();
        let mut viz = WorkflowVisualization::new(workflow);

        viz.handle_event(WorkflowEvent::Started {
            run_id: "run-1".to_string(),
            workflow_name: "test".to_string(),
        });
        assert_eq!(viz.status, WorkflowStatus::Running);

        viz.handle_event(WorkflowEvent::StepStarted {
            step_name: "start".to_string(),
        });
        assert_eq!(viz.nodes["start"].status, NodeStatus::Running);

        viz.handle_event(WorkflowEvent::StepCompleted {
            step_name: "start".to_string(),
            duration_ms: 100,
        });
        assert_eq!(viz.nodes["start"].status, NodeStatus::Completed);
        assert_eq!(viz.nodes["start"].duration_ms, Some(100));
    }

    #[test]
    fn test_progress_calculation() {
        let workflow = create_test_workflow();
        let mut viz = WorkflowVisualization::new(workflow);

        assert_eq!(viz.progress(), 0.0);

        viz.nodes.get_mut("start").unwrap().status = NodeStatus::Completed;
        assert_eq!(viz.progress(), 50.0);

        viz.nodes.get_mut("end").unwrap().status = NodeStatus::Completed;
        assert_eq!(viz.progress(), 100.0);
    }
}
