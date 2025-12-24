# Tool Output Format for GUI/TUI Integration

## Overview

AOF tools should return **structured, semantic output** that can be rendered beautifully in:
- CLI (plain text, tables, JSON)
- TUI (interactive terminal UI with ratatui)
- Web/Desktop GUI (React components, charts, graphs)

## Design Principles

1. **Semantic Structure**: Output contains meaning, not just text
2. **Render Hints**: Metadata about how to visualize data
3. **Multi-Format**: Same data renders differently per context
4. **Progressive Enhancement**: Works in plain text, better in GUI
5. **Action Support**: Tools can suggest next actions

## Output Format

```rust
pub struct ToolOutput {
    /// Raw output for backward compatibility
    pub text: String,

    /// Structured data (JSON-serializable)
    pub data: Option<serde_json::Value>,

    /// How to render this output
    pub render_hints: RenderHints,

    /// Suggested actions user can take
    pub actions: Vec<SuggestedAction>,

    /// Status/severity
    pub status: OutputStatus,
}

pub struct RenderHints {
    /// Preferred visualization type
    pub viz_type: VisualizationType,

    /// Table configuration (if viz_type is Table)
    pub table: Option<TableConfig>,

    /// Chart configuration (if viz_type is Chart)
    pub chart: Option<ChartConfig>,

    /// Metrics for dashboard cards
    pub metrics: Vec<MetricCard>,

    /// Alerts/warnings to highlight
    pub alerts: Vec<Alert>,
}

pub enum VisualizationType {
    Table,          // Tabular data
    Chart,          // Time series, bar chart, pie chart
    Metrics,        // Dashboard-style metric cards
    Tree,           // Hierarchical data
    Json,           // Raw JSON viewer
    Logs,           // Log stream viewer
    Diff,           // Side-by-side diff
    Timeline,       // Event timeline
    Network,        // Network graph
}

pub struct TableConfig {
    pub columns: Vec<TableColumn>,
    pub sortable: bool,
    pub filterable: bool,
    pub row_colors: Option<Vec<(String, String)>>,  // condition -> color
}

pub struct ChartConfig {
    pub chart_type: ChartType,
    pub x_axis: String,
    pub y_axis: Vec<String>,
    pub time_series: bool,
}

pub enum ChartType {
    Line,
    Bar,
    Pie,
    Area,
    Scatter,
}

pub struct MetricCard {
    pub label: String,
    pub value: String,
    pub unit: Option<String>,
    pub trend: Option<Trend>,        // up/down/stable
    pub threshold: Option<Threshold>, // normal/warning/critical
    pub sparkline: Option<Vec<f64>>,  // Mini chart
}

pub struct Alert {
    pub level: AlertLevel,  // info, warning, error, critical
    pub message: String,
    pub details: Option<String>,
    pub action: Option<SuggestedAction>,
}

pub struct SuggestedAction {
    pub label: String,
    pub command: String,
    pub description: String,
    pub confirm: bool,  // Requires confirmation
}
```

## Examples

### Example 1: docker_stats

**Structured Output:**
```json
{
  "text": "Container stats retrieved successfully",
  "data": {
    "containers": [
      {
        "id": "b222dede438d",
        "name": "kind-worker",
        "cpu_percent": 16.40,
        "memory_usage": 2487000000,
        "memory_limit": 9705000000,
        "memory_percent": 25.63,
        "net_io": {"rx": 4950000000, "tx": 8270000000},
        "block_io": {"read": 6170000000, "write": 4960000000},
        "pids": 848
      }
    ],
    "summary": {
      "total_containers": 17,
      "running": 9,
      "stopped": 8,
      "total_cpu": 47.53,
      "total_memory_gb": 4.8
    }
  },
  "render_hints": {
    "viz_type": "Metrics",
    "metrics": [
      {
        "label": "Running Containers",
        "value": "9",
        "threshold": "normal"
      },
      {
        "label": "Total CPU Usage",
        "value": "47.53",
        "unit": "%",
        "threshold": "warning"
      },
      {
        "label": "Total Memory",
        "value": "4.8",
        "unit": "GB",
        "threshold": "normal"
      }
    ],
    "table": {
      "columns": [
        {"name": "Container", "field": "name", "width": 20},
        {"name": "CPU %", "field": "cpu_percent", "align": "right"},
        {"name": "Memory", "field": "memory_percent", "align": "right"},
        {"name": "Net I/O", "field": "net_io", "format": "bytes"}
      ],
      "row_colors": [
        {"condition": "cpu_percent > 80", "color": "red"},
        {"condition": "memory_percent > 80", "color": "yellow"}
      ]
    },
    "alerts": [
      {
        "level": "warning",
        "message": "kind-worker using 25.63% memory",
        "action": {
          "label": "Check logs",
          "command": "docker logs kind-worker --tail 50"
        }
      }
    ]
  },
  "actions": [
    {
      "label": "Stop high CPU containers",
      "command": "docker stop $(docker ps --filter 'cpu>50' -q)",
      "confirm": true
    },
    {
      "label": "View logs",
      "command": "docker logs {container_id} --tail 100"
    }
  ],
  "status": "success"
}
```

**CLI Rendering:**
```
Container Stats (9 running, 8 stopped)

NAME                    CPU%    MEMORY%    NET I/O         BLOCK I/O       PIDS
kind-worker            16.40%   25.63%     4.95GB/8.27GB   6.17GB/4.96GB   848  ⚠️
kind-worker2           18.29%   15.49%     10.3GB/1.94GB   3.64GB/8.33GB   595
kind-control-plane     11.14%   13.61%     484MB/5.55GB    1.19GB/7.02GB   339

⚠️  WARNING: kind-worker using high memory (25.63%)
    → Run: docker logs kind-worker --tail 50

Actions:
  1. Stop high CPU containers
  2. View container logs
```

**TUI Rendering (ratatui):**
```
┌─ Container Health ───────────────────────────────────────────┐
│ ● 9 Running  ○ 8 Stopped  ⚠️ 1 Warning                       │
├──────────────────────────────────────────────────────────────┤
│ CPU: 47.5% ▓▓▓▓▓▓▓░░░  Memory: 4.8GB ▓▓▓▓▓░░░░░  PIDs: 2888│
└──────────────────────────────────────────────────────────────┘

┌─ Containers (sortable: ↑/↓) ─────────────────────────────────┐
│ NAME                CPU%    MEM%    NET I/O      PIDS  Status│
├──────────────────────────────────────────────────────────────┤
│ kind-worker        16.40%  25.63%  ↑4.95GB      848   🟡 WARN│
│ kind-worker2       18.29%  15.49%  ↑10.3GB      595   🟢 OK  │
│ kind-control-plane 11.14%  13.61%  ↑484MB       339   🟢 OK  │
└──────────────────────────────────────────────────────────────┘

⚠️  kind-worker: High memory usage (25.63%)
    [L] View Logs  [R] Restart  [S] Stop  [Enter] Details
```

**Web/Desktop GUI (React):**
```tsx
<ContainerDashboard>
  <MetricsRow>
    <MetricCard
      label="Running Containers"
      value={9}
      trend="stable"
      icon={<Container />}
    />
    <MetricCard
      label="Total CPU"
      value="47.5%"
      trend="up"
      threshold="warning"
      sparkline={[30, 35, 42, 45, 47.5]}
    />
    <MetricCard
      label="Total Memory"
      value="4.8 GB"
      trend="stable"
    />
  </MetricsRow>

  <Alert severity="warning">
    kind-worker using high memory (25.63%)
    <Button onClick={() => viewLogs('kind-worker')}>
      View Logs
    </Button>
  </Alert>

  <DataTable
    data={containers}
    columns={[...]}
    sortable
    filterable
    rowColors={{
      warning: (row) => row.cpu_percent > 80,
      error: (row) => row.memory_percent > 90
    }}
    onRowClick={(container) => showDetails(container)}
  />

  <ChartPanel>
    <LineChart
      data={cpuHistory}
      title="CPU Usage Over Time"
      xAxis="time"
      yAxis={["cpu_percent"]}
    />
  </ChartPanel>
</ContainerDashboard>
```

### Example 2: kubectl get pods

**Structured Output:**
```json
{
  "data": {
    "pods": [
      {
        "name": "nginx-deployment-7d64c8d9f5-abc12",
        "namespace": "default",
        "status": "Running",
        "ready": "1/1",
        "restarts": 0,
        "age": "2d",
        "conditions": [
          {"type": "Ready", "status": "True"},
          {"type": "ContainersReady", "status": "True"}
        ],
        "health_score": 100
      },
      {
        "name": "broken-pod-xyz",
        "namespace": "default",
        "status": "CrashLoopBackOff",
        "ready": "0/1",
        "restarts": 15,
        "age": "1h",
        "health_score": 20
      }
    ]
  },
  "render_hints": {
    "viz_type": "Table",
    "table": {
      "columns": [...],
      "row_colors": [
        {"condition": "status == 'CrashLoopBackOff'", "color": "red"},
        {"condition": "restarts > 5", "color": "yellow"}
      ]
    },
    "alerts": [
      {
        "level": "error",
        "message": "Pod broken-pod-xyz in CrashLoopBackOff",
        "action": {
          "label": "View logs",
          "command": "kubectl logs broken-pod-xyz"
        }
      }
    ]
  },
  "actions": [
    {
      "label": "Describe failing pod",
      "command": "kubectl describe pod broken-pod-xyz"
    },
    {
      "label": "Delete and recreate",
      "command": "kubectl delete pod broken-pod-xyz",
      "confirm": true
    }
  ]
}
```

## Implementation Strategy

### Phase 1: Core Infrastructure (v0.4.0)
- Add `ToolOutput` struct to aof-core
- Update `ToolResult` to include structured output
- Add rendering traits for CLI/TUI/GUI

### Phase 2: Tool Migration (v0.5.0)
- Update docker tools with structured output
- Update kubectl tools with structured output
- Add git tools with diff visualization

### Phase 3: TUI Implementation (v0.6.0)
- Build ratatui-based TUI
- Interactive table views
- Chart rendering with tui-charts
- Log streaming viewer

### Phase 4: Web/Desktop GUI (v1.0.0)
- React component library
- Real-time updates via WebSocket
- Chart.js/Recharts integration
- Action buttons and workflows

## Benefits

### 1. **Flexibility**
- Same tool works everywhere: CLI, TUI, Web, Desktop
- Rendering adapts to context automatically

### 2. **Rich UX**
- Metrics cards instead of text
- Interactive tables with sort/filter
- Charts and graphs for trends
- Color-coded alerts

### 3. **Actionable**
- Suggested next actions
- One-click commands
- Workflow automation

### 4. **Backward Compatible**
- `text` field always present for plain output
- Tools work without GUI

### 5. **LLM-Friendly**
- Structured data easier for LLM to parse
- Clear semantic meaning
- Better reasoning about health/status

## File Structure

```
crates/aof-core/src/
  tool/
    output.rs         - ToolOutput struct
    render.rs         - Rendering traits
    visualize.rs      - Visualization configs

crates/aof-tools/src/
  tools/
    docker.rs         - Updated with structured output
    kubectl.rs        - Updated with structured output

crates/aof-tui/       - NEW: Terminal UI
  src/
    widgets/
      table.rs
      metrics.rs
      chart.rs
      logs.rs

crates/aof-gui/       - Future: Web/Desktop GUI
  ui/
    src/
      components/
        ToolOutput.tsx
        MetricCard.tsx
        DataTable.tsx
```

## Next Steps

1. Implement `ToolOutput` in aof-core
2. Update docker_stats to return structured output
3. Create rendering system for CLI
4. Document for other tool developers
