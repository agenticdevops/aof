use anyhow::{Context, Result};
use aof_core::AgentConfig;
use aof_runtime::Runtime;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::mpsc as tokio_mpsc;
use tracing::info;
use crate::resources::ResourceType;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap, Gauge, Scrollbar, ScrollbarOrientation, ScrollbarState},
    text::{Line, Span},
    style::{Modifier, Color, Style},
    layout::{Layout, Direction, Alignment, Constraint},
};
use std::time::Instant;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Log writer that sends log lines to an mpsc channel
struct LogWriter(Arc<Mutex<Sender<String>>>);

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let text = String::from_utf8_lossy(buf);
        for line in text.lines() {
            if !line.is_empty() {
                let _ = self.0.lock().unwrap().send(line.to_string());
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Execute a resource (agent, workflow, job) with configuration and input
pub async fn execute(
    resource_type: &str,
    name_or_config: &str,
    input: Option<&str>,
    output: &str,
) -> Result<()> {
    // Parse resource type
    let rt = ResourceType::from_str(resource_type)
        .ok_or_else(|| anyhow::anyhow!("Unknown resource type: {}", resource_type))?;

    match rt {
        ResourceType::Agent => run_agent(name_or_config, input, output).await,
        ResourceType::Workflow | ResourceType::Flow => run_workflow(name_or_config, input, output).await,
        ResourceType::Fleet => run_fleet(name_or_config, input, output).await,
        ResourceType::Job => run_job(name_or_config, input, output).await,
        _ => {
            anyhow::bail!("Resource type '{}' cannot be run directly", resource_type)
        }
    }
}

/// Run an agent with configuration
async fn run_agent(config: &str, input: Option<&str>, output: &str) -> Result<()> {
    // Check if interactive mode should be enabled (when no input provided and stdin is a TTY)
    let interactive = input.is_none() && io::stdin().is_terminal();

    if interactive {
        // Load agent configuration
        let config_content = fs::read_to_string(config)
            .with_context(|| format!("Failed to read config file: {}", config))?;

        let agent_config: AgentConfig = serde_yaml::from_str(&config_content)
            .with_context(|| format!("Failed to parse agent config from: {}", config))?;

        let agent_name = agent_config.name.clone();

        // Create runtime and load agent
        let mut runtime = Runtime::new();
        runtime
            .load_agent_from_config(agent_config)
            .await
            .context("Failed to load agent")?;

        // Launch interactive REPL mode with TUI log capture
        run_agent_interactive(&runtime, &agent_name, output).await?;
        return Ok(());
    }

    // Non-interactive mode: normal logging to console
    info!("Loading agent config from: {}", config);

    let config_content = fs::read_to_string(config)
        .with_context(|| format!("Failed to read config file: {}", config))?;

    let agent_config: AgentConfig = serde_yaml::from_str(&config_content)
        .with_context(|| format!("Failed to parse agent config from: {}", config))?;

    let agent_name = agent_config.name.clone();
    info!("Agent loaded: {}", agent_name);

    // Create runtime and load agent
    let mut runtime = Runtime::new();
    runtime
        .load_agent_from_config(agent_config)
        .await
        .context("Failed to load agent")?;

    // Single execution mode
    let input_str = input.unwrap_or("default input");
    let result = runtime
        .execute(&agent_name, input_str)
        .await
        .context("Failed to execute agent")?;

    // Output result in requested format
    output_result(&agent_name, &result, output)?;

    Ok(())
}

/// Application state for TUI
struct AppState {
    chat_history: Vec<(String, String)>, // (role, message)
    current_input: String,
    logs: Vec<String>,
    agent_busy: bool,
    last_error: Option<String>,
    execution_start: Option<Instant>,
    execution_time_ms: u128,
    message_count: usize,
    spinner_state: u8,
    log_receiver: Receiver<String>,
    model_name: String,
    tools: Vec<String>,
    execution_result_rx: tokio_mpsc::Receiver<Result<String, String>>,
    input_tokens: u32,
    output_tokens: u32,
    context_window: u32, // Max context window for model
    chat_scroll_offset: u16, // Scroll offset for chat history
}

impl AppState {
    fn new(log_receiver: Receiver<String>, model_name: String, tools: Vec<String>) -> Self {
        let (tx, rx) = tokio_mpsc::channel(1);
        let _ = tx; // Drop sender since we only use the receiver

        // Set context window based on model
        let context_window = match model_name.as_str() {
            "google:gemini-2.5-flash" => 1000000, // 1M tokens
            "google:gemini-2.0-flash" => 1000000,
            "openai:gpt-4-turbo" => 128000,
            "openai:gpt-4" => 8192,
            _ => 128000, // default
        };

        // Create greeting message with ASCII art
        let greeting = r#"
████╗  ████╗ ████████╗
██╔═██╗██╔═██╗██╔═════╝
██████║██║ ██║█████╗
██╔═██║██║ ██║██╔══╝
██║ ██║╚████╔╝██║
╚═╝ ╚═╝ ╚═══╝ ╚═╝

Agentic Ops Framework
aof.sh"#;

        let mut chat_history = Vec::new();
        chat_history.push(("system".to_string(), greeting.to_string()));

        Self {
            chat_history,
            current_input: String::new(),
            logs: Vec::new(),
            agent_busy: false,
            last_error: None,
            execution_start: None,
            execution_time_ms: 0,
            message_count: 0,
            spinner_state: 0,
            log_receiver,
            model_name,
            tools,
            execution_result_rx: rx,
            input_tokens: 0,
            output_tokens: 0,
            context_window,
            chat_scroll_offset: 0,
        }
    }

    fn consume_logs(&mut self) {
        // Drain all available logs from the receiver (non-blocking)
        while let Ok(log) = self.log_receiver.try_recv() {
            // Keep only last 1000 logs to avoid memory bloat
            if self.logs.len() >= 1000 {
                self.logs.remove(0);
            }
            self.logs.push(log);
        }
    }

    fn update_execution_time(&mut self) {
        if let Some(start) = self.execution_start {
            self.execution_time_ms = start.elapsed().as_millis();
        }
    }

    fn next_spinner(&mut self) {
        self.spinner_state = (self.spinner_state + 1) % 4;
    }

    fn get_spinner(&self) -> &str {
        match self.spinner_state {
            0 => "◐",
            1 => "◓",
            2 => "◑",
            3 => "◒",
            _ => "◐",
        }
    }

    fn update_token_count(&mut self, text: &str) {
        // Rough estimate: ~4 characters per token
        let estimated_tokens = (text.len() / 4) as u32;
        self.output_tokens = self.output_tokens.saturating_add(estimated_tokens);
    }

    fn scroll_up(&mut self, amount: u16) {
        self.chat_scroll_offset = self.chat_scroll_offset.saturating_add(amount);
    }

    fn scroll_down(&mut self, amount: u16) {
        self.chat_scroll_offset = self.chat_scroll_offset.saturating_sub(amount);
    }

    fn auto_scroll_to_bottom(&mut self) {
        self.chat_scroll_offset = 0;
    }
}

/// Run agent in interactive REPL mode with two-column TUI
async fn run_agent_interactive(runtime: &Runtime, agent_name: &str, _output: &str) -> Result<()> {
    // Extract model and tools from runtime
    let model_name = runtime
        .get_agent(agent_name)
        .map(|agent| agent.config().model.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let tools: Vec<String> = runtime
        .get_agent(agent_name)
        .map(|agent| agent.config().tool_names().iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();

    // Create log channel
    let (log_tx, log_rx) = channel::<String>();

    // Setup tracing to capture logs into the channel instead of stdout
    let log_tx_clone = Arc::new(Mutex::new(log_tx));
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(move || LogWriter(log_tx_clone.clone()))
        .with_ansi(false)
        .with_level(true)
        .with_target(true);

    // Initialize tracing with the LogWriter layer (no console output)
    // In interactive mode, main.rs skips tracing entirely, so we initialize here
    let _ = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    // Setup terminal with panic hook for proper cleanup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Setup panic hook to restore terminal on panic
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        default_hook(panic_info);
    }));

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app state with log receiver
    let mut app_state = AppState::new(log_rx, model_name, tools);
    let should_quit = Arc::new(Mutex::new(false));

    // Don't add welcome message yet - it will show after greeting is dismissed
    // app_state.chat_history.push(("system".to_string(),
    //     format!("Connected to agent: {}\nType your query and press Enter. Commands: help, exit, quit", agent_name)));

    // Draw initial screen with greeting
    terminal.draw(|f| ui(f, agent_name, &app_state))?;

    // Main loop
    loop {
        // Check for quit
        if *should_quit.lock().unwrap() {
            break;
        }

        // Handle user input (non-blocking)
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            let evt = event::read()?;
            match evt {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers == crossterm::event::KeyModifiers::CONTROL => {
                            break;
                        }
                        KeyCode::PageUp => {
                            app_state.scroll_up(5);
                        }
                        KeyCode::PageDown => {
                            app_state.scroll_down(5);
                        }
                        KeyCode::Up if key.modifiers == crossterm::event::KeyModifiers::SHIFT => {
                            app_state.scroll_up(1);
                        }
                        KeyCode::Down if key.modifiers == crossterm::event::KeyModifiers::SHIFT => {
                            app_state.scroll_down(1);
                        }
                        KeyCode::Enter => {
                        let trimmed = app_state.current_input.trim();

                        if trimmed.is_empty() {
                            // Do nothing for empty input
                        } else if trimmed.to_lowercase() == "exit" || trimmed.to_lowercase() == "quit" {
                            break;
                        } else if trimmed.to_lowercase() == "help" {
                            app_state.chat_history.push(("system".to_string(),
                                "Available: help, exit, quit. Type normally to chat with agent.".to_string()));
                        } else {
                            // Execute agent with timer updates during execution
                            app_state.chat_history.push(("user".to_string(), trimmed.to_string()));
                            // Update input tokens based on user query length
                            let input_tokens_estimate = (trimmed.len() / 4) as u32;
                            app_state.input_tokens = app_state.input_tokens.saturating_add(input_tokens_estimate);
                            app_state.agent_busy = true;
                            app_state.last_error = None;
                            app_state.execution_start = Some(Instant::now());
                            app_state.message_count = app_state.chat_history.len();

                            // Draw busy state before execution
                            terminal.draw(|f| ui(f, agent_name, &app_state))?;

                            // Execute with periodic UI updates using select! for timer
                            let input_str = trimmed.to_string();
                            let mut exec_future = Box::pin(runtime.execute(agent_name, &input_str));
                            let mut timer_handle = tokio::time::interval(std::time::Duration::from_millis(100));

                            loop {
                                tokio::select! {
                                    result = &mut exec_future => {
                                        match result {
                                            Ok(response) => {
                                                if response.is_empty() {
                                                    let error_msg = "Error: Empty response from agent".to_string();
                                                    app_state.chat_history.push(("error".to_string(), error_msg.clone()));
                                                    app_state.last_error = Some(error_msg);
                                                } else {
                                                    // Update output tokens based on response length
                                                    app_state.update_token_count(&response);
                                                    app_state.chat_history.push(("assistant".to_string(), response));
                                                    // Auto-scroll to latest message
                                                    app_state.auto_scroll_to_bottom();
                                                }
                                            }
                                            Err(e) => {
                                                let error_msg = format!("Error: {}", e);
                                                app_state.chat_history.push(("error".to_string(), error_msg.clone()));
                                                app_state.last_error = Some(error_msg);
                                            }
                                        }
                                        app_state.agent_busy = false;
                                        app_state.update_execution_time();
                                        break;
                                    }
                                    _ = timer_handle.tick() => {
                                        // Update timer and spinner while execution is happening
                                        app_state.next_spinner();
                                        app_state.update_execution_time();

                                        // Consume any new logs
                                        app_state.consume_logs();

                                        // Redraw to show timer updates
                                        terminal.draw(|f| ui(f, agent_name, &app_state))?;
                                    }
                                }
                            }
                        }

                        app_state.current_input.clear();
                    }
                    KeyCode::Backspace => {
                        app_state.current_input.pop();
                    }
                    KeyCode::Char(c) => {
                        app_state.current_input.push(c);
                    }
                    _ => {}
                    }
                }
                Event::Mouse(mouse) => {
                    use crossterm::event::MouseEventKind;
                    match mouse.kind {
                        MouseEventKind::ScrollUp => {
                            app_state.scroll_down(3);
                        }
                        MouseEventKind::ScrollDown => {
                            app_state.scroll_up(3);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Update animation state for spinner during idle time
        if app_state.agent_busy {
            app_state.next_spinner();
            app_state.update_execution_time();
        }

        // Consume any new log messages from the channel
        app_state.consume_logs();

        // Redraw UI
        terminal.draw(|f| ui(f, agent_name, &app_state))?;
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    println!("\n-- Exiting Agentic Ops Framework --\n");
    Ok(())
}

/// Render the TUI with elegant professional styling for DevOps engineers
fn ui(f: &mut Frame, agent_name: &str, app: &AppState) {
    let tools_str = if app.tools.is_empty() {
        "none".to_string()
    } else {
        app.tools.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
    };

    // Minimalist black and white color scheme
    let primary_white = Color::White;

    // Main layout with footer for metrics
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(f.size());

    // Content area
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_layout[0]);

    // Left panel - Chat Interface
    let chat_block = Block::default()
        .title(Span::styled(
            format!(" {} ", agent_name.to_uppercase()),
            Style::default().fg(primary_white).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Thick)
        .border_style(Style::default().fg(primary_white))
        .padding(ratatui::widgets::Padding::symmetric(1, 0));

    let mut chat_lines = Vec::new();

    // Add conversation history
    for (role, msg) in &app.chat_history {
        let (style, prefix) = match role.as_str() {
            "user" => (
                Style::default().fg(Color::White),
                " ❯ ",
            ),
            "assistant" => (
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                " ◈ ",
            ),
            "error" => (
                Style::default().fg(Color::White),
                " ✗ ",
            ),
            _ => (
                Style::default().fg(Color::Gray),
                " ► ",
            ),
        };

        for line in msg.lines() {
            chat_lines.push(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(line, style),
            ]));
        }
        chat_lines.push(Line::from("")); // Spacing
    }

    // Input line with active indicator
    if app.agent_busy {
        let time_str = format!("{}ms", app.execution_time_ms);
        let busy_indicator = format!("{} Processing... {}", app.get_spinner(), time_str);
        chat_lines.push(Line::from(Span::styled(
            busy_indicator,
            Style::default().fg(Color::White).add_modifier(Modifier::DIM),
        )));
    } else {
        let mut input_spans = vec![Span::raw(" ❯ ")];

        // Show input with cursor
        if app.current_input.is_empty() {
            input_spans.push(Span::styled("_", Style::default().fg(Color::Gray).add_modifier(Modifier::DIM)));
        } else {
            input_spans.push(Span::raw(&app.current_input));
            input_spans.push(Span::styled("_", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
        }
        chat_lines.push(Line::from(input_spans));
    }

    // Calculate scroll position with manual scroll offset
    let visible_height = chunks[0].height.saturating_sub(3) as usize; // Account for borders and padding
    let total_lines = chat_lines.len();

    // If user hasn't manually scrolled, auto-scroll to show input at bottom
    let mut scroll_offset = if total_lines > visible_height {
        total_lines.saturating_sub(visible_height)
    } else {
        0
    };

    // Apply manual scroll offset (user scrolling up/down)
    if app.chat_scroll_offset > 0 {
        scroll_offset = scroll_offset.saturating_add(app.chat_scroll_offset as usize);
    } else if app.chat_scroll_offset == 0 && !app.agent_busy {
        // Auto-scroll to bottom when not scrolled and agent is idle
        scroll_offset = if total_lines > visible_height {
            total_lines.saturating_sub(visible_height)
        } else {
            0
        };
    }

    let chat_para = Paragraph::new(chat_lines.clone())
        .block(chat_block)
        .wrap(Wrap { trim: true })
        .scroll((scroll_offset as u16, 0));

    // Render scrollbar with state
    let mut scrollbar_state = ScrollbarState::new(total_lines)
        .position(scroll_offset);

    f.render_widget(chat_para, chunks[0]);
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight),
        chunks[0],
        &mut scrollbar_state,
    );

    // Split right panel into two rows: logs (80%) and stats (20%)
    let right_panel = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(chunks[1]);

    // Top row - System Logs
    let logs_block = Block::default()
        .title(Span::styled(
            " SYSTEM LOG ",
            Style::default().fg(primary_white).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Thick)
        .border_style(Style::default().fg(primary_white))
        .padding(ratatui::widgets::Padding::symmetric(1, 0));

    let log_lines: Vec<Line> = app.logs.iter()
        .map(|log| {
            let style = if log.contains("ERROR") {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else if log.contains("WARN") {
                Style::default().fg(Color::White)
            } else if log.contains("DEBUG") {
                Style::default().fg(Color::Gray).add_modifier(Modifier::DIM)
            } else if log.contains("INFO") {
                Style::default().fg(Color::White).add_modifier(Modifier::DIM)
            } else {
                Style::default().fg(Color::Gray)
            };

            let trimmed = log.chars().take(right_panel[0].width.saturating_sub(4) as usize).collect::<String>();
            Line::from(Span::styled(trimmed, style))
        })
        .collect();

    let logs_para = Paragraph::new(log_lines)
        .block(logs_block)
        .wrap(Wrap { trim: true })
        .scroll((
            (app.logs.len() as u16).saturating_sub(right_panel[0].height.saturating_sub(3) / 2),
            0,
        ));

    f.render_widget(logs_para, right_panel[0]);

    // Bottom row - Context Stats
    let context_used = app.input_tokens + app.output_tokens;
    let context_percentage = if app.context_window > 0 {
        (context_used as f64 / app.context_window as f64) * 100.0
    } else {
        0.0
    };

    // Create gauge for visual representation
    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(Span::styled(
                    " CONTEXT USAGE ",
                    Style::default().fg(primary_white).add_modifier(Modifier::BOLD),
                ))
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Thick)
                .border_style(Style::default().fg(primary_white))
        )
        .gauge_style(Style::default().fg(Color::Green))
        .ratio(context_percentage / 100.0)
        .label(Span::raw(format!(
            "  IN: {} │ OUT: {} │ TOTAL: {} / {} ({:.1}%)",
            app.input_tokens, app.output_tokens, context_used, app.context_window, context_percentage
        )));

    f.render_widget(gauge, right_panel[1]);

    // Footer metrics bar
    let metrics_text = if app.agent_busy {
        format!(
            "  ⧖ {:>5}ms  │  {} {} messages  │  Model: {}  │  Tools: {}  │  Status: Active",
            app.execution_time_ms,
            app.get_spinner(),
            app.message_count / 2,
            app.model_name,
            tools_str
        )
    } else {
        format!(
            "  ✓ Completed  │  {} messages  │  Model: {}  │  Tools: {}  │  Last execution: {}ms",
            app.message_count / 2,
            app.model_name,
            tools_str,
            app.execution_time_ms
        )
    };

    let metrics_block = Block::default()
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .padding(ratatui::widgets::Padding::symmetric(1, 0));

    let metrics_para = Paragraph::new(metrics_text)
        .block(metrics_block)
        .style(Style::default().fg(Color::White));

    f.render_widget(metrics_para, main_layout[1]);
}

/// Format and output agent result
fn output_result(agent_name: &str, result: &str, output: &str) -> Result<()> {
    match output {
        "json" => {
            let json_output = serde_json::json!({
                "success": true,
                "agent": agent_name,
                "result": result
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        "yaml" => {
            let yaml_output = serde_yaml::to_string(&serde_json::json!({
                "success": true,
                "agent": agent_name,
                "result": result
            }))?;
            println!("{}", yaml_output);
        }
        "text" | _ => {
            println!("Agent: {}", agent_name);
            println!("Result: {}", result);
        }
    }
    Ok(())
}

/// Run a workflow with configuration
/// Auto-detects between AgentFlow and Workflow based on YAML kind
async fn run_workflow(config_path: &str, input: Option<&str>, output: &str) -> Result<()> {
    use tokio::sync::mpsc;

    info!("Loading flow from: {}", config_path);

    // Read the config to detect kind
    let content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config: {}", config_path))?;

    // Detect kind from YAML
    let kind = detect_yaml_kind(&content);

    match kind.as_deref() {
        Some("AgentFlow") => run_agentflow(config_path, &content, input, output).await,
        Some("Workflow") | _ => run_workflow_internal(config_path, input, output).await,
    }
}

/// Detect the 'kind' field from a YAML document
fn detect_yaml_kind(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("kind:") {
            let value = trimmed.strip_prefix("kind:")?.trim();
            let value = value.trim_matches('"').trim_matches('\'');
            return Some(value.to_string());
        }
    }
    None
}

/// Run an AgentFlow (trigger-based event-driven flow)
async fn run_agentflow(config_path: &str, _content: &str, input: Option<&str>, output: &str) -> Result<()> {
    use aof_runtime::AgentFlowExecutor;
    use tokio::sync::mpsc;

    info!("Executing AgentFlow from: {}", config_path);

    // Create runtime for agent execution
    let runtime = std::sync::Arc::new(Runtime::new());

    // Create AgentFlow executor from file
    let executor = AgentFlowExecutor::from_file(config_path, runtime.clone())
        .await
        .context("Failed to load AgentFlow")?;

    // Parse trigger data from input
    let trigger_data: serde_json::Value = if let Some(inp) = input {
        serde_json::from_str(inp).unwrap_or_else(|_| {
            // If not valid JSON, create a simple event structure
            serde_json::json!({
                "event": {
                    "text": inp,
                    "user": "cli_user",
                    "channel": "cli",
                    "ts": chrono::Utc::now().timestamp().to_string()
                }
            })
        })
    } else {
        // Default manual trigger event
        serde_json::json!({
            "event": {
                "type": "manual",
                "user": "cli_user",
                "channel": "cli",
                "ts": chrono::Utc::now().timestamp().to_string()
            }
        })
    };

    // Create event channel for monitoring
    let (event_tx, mut event_rx) = mpsc::channel(100);
    let executor = executor.with_event_channel(event_tx);

    // Spawn task to print events
    let event_printer = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            use aof_runtime::AgentFlowEvent;
            match event {
                AgentFlowEvent::Started { flow_name, run_id } => {
                    eprintln!("[AGENTFLOW] Started: {} (run: {})", flow_name, run_id);
                }
                AgentFlowEvent::NodeStarted { node_id, node_type } => {
                    eprintln!("[NODE] Starting: {} ({})", node_id, node_type);
                }
                AgentFlowEvent::NodeCompleted { node_id, duration_ms, .. } => {
                    eprintln!("[NODE] Completed: {} ({}ms)", node_id, duration_ms);
                }
                AgentFlowEvent::NodeFailed { node_id, error } => {
                    eprintln!("[NODE] Failed: {} - {}", node_id, error);
                }
                AgentFlowEvent::Waiting { node_id, reason } => {
                    eprintln!("[NODE] Waiting: {} - {}", node_id, reason);
                }
                AgentFlowEvent::VariableSet { key, value } => {
                    eprintln!("[VAR] Set: {} = {}", key, value);
                }
                AgentFlowEvent::Completed { run_id, status } => {
                    eprintln!("[AGENTFLOW] Completed: {} with status {:?}", run_id, status);
                }
                AgentFlowEvent::Error { message } => {
                    eprintln!("[ERROR] {}", message);
                }
            }
        }
    });

    // Execute the AgentFlow
    let final_state = executor.execute(trigger_data).await
        .context("AgentFlow execution failed")?;

    // Wait for event printer to finish
    drop(executor);
    let _ = event_printer.await;

    // Get completed node names from node_results
    let completed_nodes: Vec<String> = final_state
        .node_results
        .iter()
        .filter(|(_, r)| r.status == aof_core::NodeExecutionStatus::Completed)
        .map(|(id, _)| id.clone())
        .collect();

    // Output result in requested format
    match output {
        "json" => {
            let json_output = serde_json::json!({
                "success": true,
                "flow": final_state.flow_name,
                "run_id": final_state.run_id,
                "status": format!("{:?}", final_state.status),
                "completed_nodes": completed_nodes,
                "variables": final_state.variables,
                "node_results": final_state.node_results,
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        "yaml" => {
            let yaml_output = serde_yaml::to_string(&serde_json::json!({
                "success": true,
                "flow": final_state.flow_name,
                "run_id": final_state.run_id,
                "status": format!("{:?}", final_state.status),
                "completed_nodes": completed_nodes,
                "variables": final_state.variables,
            }))?;
            println!("{}", yaml_output);
        }
        "text" | _ => {
            println!("AgentFlow: {}", final_state.flow_name);
            println!("Run ID: {}", final_state.run_id);
            println!("Status: {:?}", final_state.status);
            if !completed_nodes.is_empty() {
                println!("Completed Nodes: {}", completed_nodes.join(" -> "));
            }
            if let Some(error) = &final_state.error {
                println!("Error: {}", error.message);
            }
        }
    }

    Ok(())
}

/// Run a traditional Workflow (step-based sequential flow)
async fn run_workflow_internal(config_path: &str, input: Option<&str>, output: &str) -> Result<()> {
    use aof_runtime::WorkflowExecutor;
    use tokio::sync::mpsc;

    info!("Loading workflow from: {}", config_path);

    // Create runtime for agent execution within workflow
    let runtime = std::sync::Arc::new(Runtime::new());

    // Create workflow executor from file
    let mut executor = WorkflowExecutor::from_file(config_path, runtime.clone())
        .await
        .context("Failed to load workflow")?;

    // Parse initial state from input
    let initial_state: serde_json::Value = if let Some(inp) = input {
        serde_json::from_str(inp).unwrap_or_else(|_| {
            // If not valid JSON, wrap as input field
            serde_json::json!({ "input": inp })
        })
    } else {
        serde_json::json!({})
    };

    // Create event channel for monitoring
    let (event_tx, mut event_rx) = mpsc::channel(100);
    let executor = executor.with_event_channel(event_tx);

    // Spawn task to print events
    let event_printer = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                aof_runtime::WorkflowEvent::Started { run_id, workflow_name } => {
                    eprintln!("[WORKFLOW] Started: {} (run: {})", workflow_name, run_id);
                }
                aof_runtime::WorkflowEvent::StepStarted { step_name } => {
                    eprintln!("[STEP] Starting: {}", step_name);
                }
                aof_runtime::WorkflowEvent::StepCompleted { step_name, duration_ms } => {
                    eprintln!("[STEP] Completed: {} ({}ms)", step_name, duration_ms);
                }
                aof_runtime::WorkflowEvent::StepFailed { step_name, error } => {
                    eprintln!("[STEP] Failed: {} - {}", step_name, error);
                }
                aof_runtime::WorkflowEvent::WaitingApproval { step_name, approvers } => {
                    eprintln!("[APPROVAL] Waiting for approval at step '{}' from: {:?}", step_name, approvers);
                }
                aof_runtime::WorkflowEvent::WaitingInput { step_name, prompt } => {
                    eprintln!("[INPUT] Waiting for input at step '{}': {}", step_name, prompt);
                }
                aof_runtime::WorkflowEvent::Completed { run_id, status } => {
                    eprintln!("[WORKFLOW] Completed: {} with status {:?}", run_id, status);
                }
                aof_runtime::WorkflowEvent::Error { message } => {
                    eprintln!("[ERROR] {}", message);
                }
                _ => {}
            }
        }
    });

    // Execute workflow
    let mut executor = executor;
    let final_state = executor.execute(initial_state).await
        .context("Workflow execution failed")?;

    // Wait for event printer to finish
    drop(executor); // Drop to close event channel
    let _ = event_printer.await;

    // Output result in requested format
    match output {
        "json" => {
            let json_output = serde_json::json!({
                "success": true,
                "workflow": final_state.workflow_name,
                "run_id": final_state.run_id,
                "status": format!("{:?}", final_state.status),
                "state": final_state.data,
                "completed_steps": final_state.completed_steps,
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        "yaml" => {
            let yaml_output = serde_yaml::to_string(&serde_json::json!({
                "success": true,
                "workflow": final_state.workflow_name,
                "run_id": final_state.run_id,
                "status": format!("{:?}", final_state.status),
                "state": final_state.data,
                "completed_steps": final_state.completed_steps,
            }))?;
            println!("{}", yaml_output);
        }
        "text" | _ => {
            println!("Workflow: {}", final_state.workflow_name);
            println!("Run ID: {}", final_state.run_id);
            println!("Status: {:?}", final_state.status);
            println!("Completed Steps: {}", final_state.completed_steps.join(" -> "));
            if let Some(error) = &final_state.error {
                println!("Error: {}", error.message);
            }
        }
    }

    Ok(())
}

/// Run a job (placeholder)
async fn run_job(name_or_config: &str, input: Option<&str>, output: &str) -> Result<()> {
    println!("Run job - Not yet implemented");
    println!("Job: {}", name_or_config);
    if let Some(inp) = input {
        println!("Input: {}", inp);
    }
    println!("Output format: {}", output);
    Ok(())
}

/// Run a fleet with configuration
async fn run_fleet(config_path: &str, input: Option<&str>, output_format: &str) -> Result<()> {
    use aof_runtime::fleet::{FleetCoordinator, FleetEvent};
    use tokio::sync::mpsc;
    use std::time::Instant;
    use crate::output::FleetOutput;

    let start_time = Instant::now();
    let output = FleetOutput::new();

    // Print banner for text output
    if output_format == "text" {
        output.print_banner();
    }

    info!("Loading fleet from: {}", config_path);

    // Create fleet coordinator from file
    let mut coordinator = FleetCoordinator::from_file(config_path)
        .await
        .context("Failed to load fleet")?;

    // Get fleet info for display
    let fleet_config = coordinator.fleet();
    let fleet_name = fleet_config.metadata.name.clone();
    let agent_count = fleet_config.spec.agents.len();
    let mode = format!("{:?}", fleet_config.spec.coordination.mode);

    // Parse input
    let task_input: serde_json::Value = if let Some(inp) = input {
        serde_json::from_str(inp).unwrap_or_else(|_| serde_json::json!({ "input": inp }))
    } else {
        serde_json::json!({})
    };

    // Create event channel for monitoring
    let (event_tx, mut event_rx) = mpsc::channel(100);
    let coordinator = coordinator.with_event_channel(event_tx);

    // Start the fleet
    let mut coordinator = coordinator;
    coordinator.start().await.context("Failed to start fleet")?;

    // Collect events in background with beautiful output
    let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let events_clone = events.clone();
    let use_pretty = output_format == "text";
    let mode_clone = mode.clone();

    let event_collector = tokio::spawn(async move {
        let output = FleetOutput::new();
        let mut tier_start = Instant::now();
        let mut tier_results = 0;

        while let Some(event) = event_rx.recv().await {
            if use_pretty {
                match &event {
                    FleetEvent::Started { fleet_name, agent_count } => {
                        output.print_fleet_header(fleet_name, *agent_count, &mode_clone);
                    }
                    FleetEvent::AgentStarted { agent_name, .. } => {
                        output.print_agent_executing(agent_name, "");
                    }
                    FleetEvent::TaskSubmitted { task_id } => {
                        output.print_task_submitted(task_id);
                    }
                    FleetEvent::TaskAssigned { agent_name, .. } => {
                        output.print_info(&format!("Task assigned to {}", agent_name));
                    }
                    FleetEvent::TaskCompleted { task_id, duration_ms } => {
                        output.print_success(&format!("Task {} completed in {}ms", &task_id[..8.min(task_id.len())], duration_ms));
                        tier_results += 1;
                    }
                    FleetEvent::TaskFailed { task_id, error } => {
                        output.print_error(&format!("Task {} failed: {}", &task_id[..8.min(task_id.len())], error));
                    }
                    FleetEvent::TierStarted { tier, agents, consensus } => {
                        tier_start = Instant::now();
                        tier_results = 0;
                        let agent_names: Vec<String> = agents.iter().map(|a| a.name.clone()).collect();
                        output.print_tier_start(*tier, &agent_names, consensus);
                    }
                    FleetEvent::TierCompleted { tier, confidence, .. } => {
                        let duration = tier_start.elapsed().as_millis() as u64;
                        output.print_tier_complete(*tier, tier_results, *confidence, duration);
                    }
                    FleetEvent::ConsensusReached { votes, .. } => {
                        output.print_success(&format!("Consensus reached with {} votes", votes));
                    }
                    FleetEvent::Stopped { .. } => {
                        // Will print completion summary separately
                    }
                    FleetEvent::Error { message } => {
                        output.print_error(message);
                    }
                    _ => {}
                }
            }
            events_clone.lock().unwrap().push(event);
        }
    });

    // Submit task
    let task_id = coordinator
        .submit_task(task_input.clone())
        .await
        .context("Failed to submit task")?;

    // Execute the task (execute_next processes the queued task)
    let task_result = coordinator
        .execute_next()
        .await
        .context("Failed to execute task")?;

    let result = task_result
        .map(|t| t.result.unwrap_or_default())
        .unwrap_or_default();

    // Stop fleet
    coordinator.stop().await.context("Failed to stop fleet")?;

    // Wait briefly for events to be collected
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    drop(event_collector);

    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Output result
    match output_format {
        "json" => {
            let output_json = serde_json::json!({
                "task_id": task_id,
                "status": "Completed",
                "result": result,
                "duration_ms": duration_ms,
                "fleet": fleet_name,
            });
            println!("{}", serde_json::to_string_pretty(&output_json)?);
        }
        "yaml" => {
            let output_yaml = serde_json::json!({
                "task_id": task_id,
                "status": "Completed",
                "result": result,
                "duration_ms": duration_ms,
                "fleet": fleet_name,
            });
            println!("{}", serde_yaml::to_string(&output_yaml)?);
        }
        "text" | _ => {
            // Print beautiful RCA report
            output.print_rca_report(&result);
            output.print_fleet_complete(&fleet_name, duration_ms, None);
        }
    }

    Ok(())
}
