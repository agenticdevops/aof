# Token Efficiency Benchmark Report

**Generated:** 2025-12-24 04:38:40 UTC

## Configuration

- Simulate retries: true
- Unified retry rate: 13.0%
- Per-op retry rate: 2.0%
- Simulate parallel: true

## Summary

| Scenario | Tasks | Unified | Per-Op | Winner | Difference |
|----------|-------|---------|--------|--------|------------|
| simple_container_list | 1 | 795 | 3329 | ✅ Unified | -318.7% |
| medium_container_debug | 2 | 1086 | 3907 | ✅ Unified | -259.8% |
| complex_full_troubleshooting | 5 | 2040 | 4245 | ✅ Unified | -108.1% |
| long_session_monitoring | 20 | 6729 | 8029 | ✅ Unified | -19.3% |
| parallel_health_check | 1 | 795 | 3329 | ✅ Unified | -318.7% |

## Detailed Results

### 1. simple_container_list

**Description:** List running containers - single tool call

**Tasks:** 1

#### Token Breakdown

| Component | Unified | Per-Op | Difference |
|-----------|---------|--------|------------|
| Tool Definitions | 404 | 3000 | +2596 |
| System Prompt | 100 | 100 | +0 |
| Reasoning | 30 | 40 | +10 |
| Tool Calls | 11 | 9 | -2 |
| Tool Outputs | 200 | 150 | -50 |
| LLM Responses | 50 | 30 | -20 |
| Retry Overhead | 0 | 0 | +0 |
| **TOTAL** | **795** | **3329** | **+2534** |

#### Execution Metrics

| Metric | Unified | Per-Op |
|--------|---------|--------|
| Success Rate | 100.0% | 100.0% |
| Retries | 0 | 0 |
| Errors | 0 | 0 |
| Parallel Calls | 0 | 0 |

### 2. medium_container_debug

**Description:** Debug container issues - multiple tool calls

**Tasks:** 2

#### Token Breakdown

| Component | Unified | Per-Op | Difference |
|-----------|---------|--------|------------|
| Tool Definitions | 404 | 3000 | +2596 |
| System Prompt | 100 | 100 | +0 |
| Reasoning | 60 | 120 | +60 |
| Tool Calls | 22 | 27 | +5 |
| Tool Outputs | 400 | 300 | -100 |
| LLM Responses | 100 | 60 | -40 |
| Retry Overhead | 0 | 300 | +300 |
| **TOTAL** | **1086** | **3907** | **+2821** |

#### Execution Metrics

| Metric | Unified | Per-Op |
|--------|---------|--------|
| Success Rate | 100.0% | 50.0% |
| Retries | 0 | 1 |
| Errors | 0 | 1 |
| Parallel Calls | 0 | 2 |

### 3. complex_full_troubleshooting

**Description:** Full container troubleshooting - many operations

**Tasks:** 5

#### Token Breakdown

| Component | Unified | Per-Op | Difference |
|-----------|---------|--------|------------|
| Tool Definitions | 404 | 3000 | +2596 |
| System Prompt | 100 | 100 | +0 |
| Reasoning | 180 | 200 | +20 |
| Tool Calls | 66 | 45 | -21 |
| Tool Outputs | 1000 | 750 | -250 |
| LLM Responses | 250 | 150 | -100 |
| Retry Overhead | 40 | 0 | -40 |
| **TOTAL** | **2040** | **4245** | **+2205** |

#### Execution Metrics

| Metric | Unified | Per-Op |
|--------|---------|--------|
| Success Rate | 80.0% | 100.0% |
| Retries | 1 | 0 |
| Errors | 1 | 0 |
| Parallel Calls | 0 | 2 |

### 4. long_session_monitoring

**Description:** Long monitoring session - 20+ operations

**Tasks:** 20

#### Token Breakdown

| Component | Unified | Per-Op | Difference |
|-----------|---------|--------|------------|
| Tool Definitions | 404 | 3000 | +2596 |
| System Prompt | 100 | 100 | +0 |
| Reasoning | 750 | 840 | +90 |
| Tool Calls | 275 | 189 | -86 |
| Tool Outputs | 4000 | 3000 | -1000 |
| LLM Responses | 1000 | 600 | -400 |
| Retry Overhead | 200 | 300 | +100 |
| **TOTAL** | **6729** | **8029** | **+1300** |

#### Execution Metrics

| Metric | Unified | Per-Op |
|--------|---------|--------|
| Success Rate | 75.0% | 95.0% |
| Retries | 5 | 1 |
| Errors | 5 | 1 |
| Parallel Calls | 0 | 0 |

### 5. parallel_health_check

**Description:** Health check across services - parallelizable

**Tasks:** 1

#### Token Breakdown

| Component | Unified | Per-Op | Difference |
|-----------|---------|--------|------------|
| Tool Definitions | 404 | 3000 | +2596 |
| System Prompt | 100 | 100 | +0 |
| Reasoning | 30 | 40 | +10 |
| Tool Calls | 11 | 9 | -2 |
| Tool Outputs | 200 | 150 | -50 |
| LLM Responses | 50 | 30 | -20 |
| Retry Overhead | 0 | 0 | +0 |
| **TOTAL** | **795** | **3329** | **+2534** |

#### Execution Metrics

| Metric | Unified | Per-Op |
|--------|---------|--------|
| Success Rate | 100.0% | 100.0% |
| Retries | 0 | 0 |
| Errors | 0 | 0 |
| Parallel Calls | 0 | 1 |

## Analysis

### Overall Statistics

- **Total tokens (all scenarios):**
  - Unified: 11445 tokens
  - Per-Op: 22839 tokens
  - Difference: +11394 tokens (+99.6%)

- **Scenarios won:**
  - Unified: 5/5
  - Per-Op: 0/5

### Key Findings

1. **Simple tasks (1 operation):** Per-Op uses +2534 tokens (+318.7%) vs Unified
   - Higher context cost dominates
   - Tool definition overhead not amortized

2. **Medium tasks (3-5 operations):** Per-Op uses +2821 tokens (+259.8%) vs Unified
   - Context cost partially amortized
   - Better accuracy reduces retry cost

3. **Long sessions (20+ operations):** Per-Op uses +1300 tokens (+19.3%) vs Unified
   - Context cost fully amortized
   - Structured output saves parsing tokens


## Recommendations

### Unified Tools Are More Token-Efficient Overall

**Use unified tools when:**
- ✅ Running one-off commands
- ✅ Short debugging sessions
- ✅ Ad-hoc exploration
- ✅ Token budget is tight

**Use per-op tools when:**
- ✅ Accuracy is critical (deployments, deletions)
- ✅ Building TUI/GUI with rich rendering
- ✅ Long monitoring sessions (20+ calls)
- ✅ Parallel execution is possible

### Best Practice: Hybrid Approach

```yaml
tools:
  # Per-op for common operations (accurate, structured)
  - docker_stats
  - docker_ps
  - kubectl_get

  # Unified for flexibility (fallback)
  - docker
  - kubectl
```

This gives you:
- ✅ Accurate per-op tools for 80% of operations
- ✅ Flexible unified tools for edge cases
- ✅ Best token efficiency AND accuracy
