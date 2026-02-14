# Technology Stack

**Analysis Date:** 2026-02-11

## Languages

**Primary:**
- **Rust** 1.75+ - All core framework crates (aof-core, aof-llm, aof-mcp, aof-runtime, aof-memory, aof-triggers, aof-tools, aof-skills)

**Secondary:**
- **Shell scripting** - Build, test, and deployment automation scripts

## Runtime

**Environment:**
- **Tokio** 1.35 - Async runtime with full features (`tokio-full`)
- **Rust Edition** 2021

**Package Manager:**
- **Cargo** - Workspace-based monorepo with 13 member crates
- **Lockfile:** `Cargo.lock` present

## Frameworks

**Core Framework:**
- **AOF (Agentic Ops Framework)** 0.4.0-beta - Apache 2.0 licensed, pure Rust framework for building agentic applications

**Runtime & Execution:**
- **aof-runtime** 0.4.0-beta - Agent task orchestration and execution engine (`crates/aof-runtime`)
- **aof-core** 0.4.0-beta - Core traits, types, abstractions (`crates/aof-core`)

**LLM Integration:**
- **aof-llm** 0.4.0-beta - Multi-provider LLM abstraction layer (`crates/aof-llm`)
  - Supported: Anthropic, OpenAI, Google, Groq, Ollama, Bedrock (optional), Azure (pending)

**Messaging & Webhooks:**
- **Axum** 0.7 - Async web framework for webhook servers
- **Tower** 0.4 - HTTP middleware and utilities
- **tower-http** 0.5 - HTTP layers (trace, CORS)

**CLI:**
- **Clap** 4.4 - CLI argument parsing with derive macros
- **ratatui** 0.26 - Terminal UI rendering
- **crossterm** 0.27 - Terminal manipulation

**External Protocols:**
- **aof-mcp** 0.4.0-beta - Model Context Protocol (MCP) client with stdio, SSE, HTTP transports

**State & Memory:**
- **aof-memory** 0.4.0-beta - Pluggable memory backends (in-memory, file-based, Redis optional, Sled optional)

**Event Triggering:**
- **aof-triggers** 0.4.0-beta - Platform-agnostic messaging triggers for webhooks
- **aof-tools** 0.4.0-beta - Modular tool implementations

**AI Skills:**
- **aof-skills** 0.4.0-beta - Skill definitions and utilities

## Key Dependencies

**Critical (Core):**
- **async-trait** 0.1 - Async trait support
- **futures** 0.3 - Future utilities and combinators
- **thiserror** 1.0 - Error handling macros
- **anyhow** 1.0 - Flexible error handling

**Serialization:**
- **serde** 1.0 with `derive` - Data serialization framework
- **serde_json** 1.0 - JSON support
- **serde_yaml** 0.9 - YAML support
- **serde_path_to_error** 0.1 - Precise error messages for config parsing

**HTTP/Networking:**
- **reqwest** 0.11 - HTTP client with JSON streaming support
- **hyper** 1.0 - HTTP protocol implementation
- **url** 2.5 - URL parsing

**Infrastructure:**
- **dashmap** 5.5 - Lock-free concurrent HashMap for state management
- **arc-swap** 1.6 - Atomic reference counting with swaps
- **parking_lot** 0.12 - Faster synchronization primitives
- **bytes** 1.5 - Efficient byte buffer handling
- **memmap2** 0.9 - Memory-mapped file support

**Utilities:**
- **uuid** 1.6 with `v4, serde` - UUID generation
- **chrono** 0.4 with `serde` - Date/time handling
- **regex** 1.10 - Pattern matching
- **rand** 0.8 - Random number generation
- **glob** 0.3 - File glob patterns
- **which** 6.0 - Executable search in PATH

**Security & Cryptography:**
- **hmac** 0.12 - HMAC signature verification
- **sha2** 0.10 - SHA-256 hashing
- **ed25519-dalek** 2.1 - EdDSA signatures
- **hex** 0.4 - Hex encoding/decoding
- **base64** 0.21 - Base64 encoding/decoding

**Rate Limiting:**
- **governor** 0.6 - Token bucket rate limiting
- **nonzero_ext** 0.3 - NonZero integer types

**Logging/Tracing:**
- **tracing** 0.1 - Structured logging
- **tracing-subscriber** 0.3 with `env-filter` - Log collection and filtering

**CLI Tools:**
- **comfy-table** 7.1 - Terminal table formatting
- **colored** 2.1 - ANSI color output
- **dirs** 5.0 - Platform directories
- **tokio-util** 0.7 - Tokio utilities
- **atty** 0.2 - TTY detection

**Testing:**
- **tempfile** 3.8 - Temporary file/directory creation
- **assert_cmd** 2.0 - CLI testing
- **predicates** 3.0 - Assertion combinators

**Optional Backends (Features):**
- **redis** 0.24 - Redis client (redis-backend feature)
- **sled** 0.34 - Embedded database (sled-backend feature)
- **aws-config** 1.0 - AWS SDK config (bedrock feature)
- **aws-sdk-bedrockruntime** 1.0 - AWS Bedrock runtime (bedrock feature)
- **aws-smithy-types** 1.3.5 - AWS Smithy types (bedrock feature)
- **async-stream** 0.3 - Async generator macros (bedrock feature)

## Build Configuration

**Release Profile:**
- **opt-level**: 3 (maximum optimization)
- **lto**: "thin" (Link-Time Optimization)
- **codegen-units**: 1 (slower compile, better optimization)
- **strip**: true (strip debug symbols for smaller binary)

**Workspace:**
- **Resolver:** 2
- **Edition:** 2021
- **MSRV:** Rust 1.75

## Platform Requirements

**Development:**
- Rust 1.75 or later
- Cargo (part of Rust installation)
- pkg-config (for native dependencies)
- libssl-dev (for TLS)

**Production:**
- Linux (Debian-based recommended per Dockerfile)
- macOS (Intel and Apple Silicon support via build)
- Windows (support added via MSRV compatibility)
- Docker support available (multi-stage build in `Dockerfile`)

## Workspace Structure

The project uses a Cargo workspace with 13 member crates:

```
crates/
├── aof-core/              # Core traits, types, abstractions
├── aof-llm/               # LLM provider abstraction (Anthropic, OpenAI, etc.)
├── aof-mcp/               # Model Context Protocol client
├── aof-runtime/           # Agent execution runtime
├── aof-memory/            # Pluggable memory backends
├── aof-triggers/          # Webhook and messaging triggers
├── aof-tools/             # Tool implementations (kubectl, docker, git, etc.)
├── aof-skills/            # AI skill definitions
├── aofctl/                # CLI binary (kubectl-style)
├── aof-viz/               # Visualization utilities
├── smoke-test-mcp/        # MCP testing
└── test-trigger-server/   # Trigger server testing
```

## Cross-Crate Dependencies

**Dependency Hierarchy:**
- `aof-core` - No internal dependencies (foundation)
- `aof-llm` - Depends on `aof-core`
- `aof-mcp` - Depends on `aof-core`
- `aof-memory` - Depends on `aof-core`
- `aof-runtime` - Depends on `aof-core, aof-mcp, aof-llm, aof-memory, aof-tools`
- `aof-triggers` - Depends on `aof-core, aof-runtime, aof-llm, aof-memory, aof-tools`
- `aofctl` - Depends on all workspace crates with all features enabled

---

*Stack analysis: 2026-02-11*
