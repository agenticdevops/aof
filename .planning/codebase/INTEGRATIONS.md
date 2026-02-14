# External Integrations

**Analysis Date:** 2026-02-11

## APIs & External Services

**LLM Providers:**
- **Anthropic** - Claude API for LLM inference
  - SDK/Client: Native implementation in `aof-llm` via `reqwest`
  - Auth: Environment variable `ANTHROPIC_API_KEY`
  - Feature: Default enabled in `aof-llm`

- **OpenAI** - GPT models for LLM inference
  - SDK/Client: Native implementation in `aof-llm` via `reqwest`
  - Auth: Environment variable `OPENAI_API_KEY`
  - Feature: Default enabled in `aof-llm`

- **Google (Gemini)** - Google AI models
  - SDK/Client: Native implementation in `aof-llm` via `reqwest`
  - Auth: `GOOGLE_API_KEY` environment variable
  - Status: Basic support

- **Groq** - Fast inference API (OpenAI-compatible)
  - SDK/Client: Uses OpenAI adapter with custom endpoint
  - Auth: Environment variable `GROQ_API_KEY`
  - Endpoint: `https://api.groq.com/openai/v1` (auto-configured)

- **Ollama** - Local LLM runtime
  - SDK/Client: Uses OpenAI adapter with custom endpoint
  - Auth: No API key required (uses placeholder "ollama")
  - Endpoint: `OLLAMA_HOST` env var (defaults to `http://localhost:11434/v1`)

- **AWS Bedrock** - AWS managed LLM service
  - SDK/Client: `aws-sdk-bedrockruntime` 1.0
  - Auth: AWS credentials via `aws-config`
  - Feature: Optional (requires `bedrock` feature flag)
  - Status: Full implementation

- **Azure** - Azure OpenAI Service
  - SDK/Client: Planned
  - Status: Not yet implemented

**Messaging Platforms:**
- **Slack** - Team chat and slash commands
  - Implementation: `SlackPlatform` in `crates/aof-triggers/src/platforms/slack.rs`
  - Config: `SlackConfig` with token and signing secret
  - Features: Message parsing, signature verification, threaded replies, ephemeral messages
  - Webhooks: URL verification, app mentions, direct messages, slash commands, interactive actions

- **Discord** - Chat and bot commands
  - Implementation: `DiscordPlatform` in `crates/aof-triggers/src/platforms/discord.rs`
  - Config: `DiscordConfig`

- **Telegram** - Messaging platform
  - Implementation: `TelegramPlatform` in `crates/aof-triggers/src/platforms/telegram.rs`
  - Config: `TelegramConfig` with bot token

- **WhatsApp** - Messaging service
  - Implementation: `WhatsAppPlatform` in `crates/aof-triggers/src/platforms/whatsapp.rs`
  - Config: `WhatsAppConfig`

- **GitHub** - Repository management and CI/CD
  - Implementation: `GitHubPlatform` in `crates/aof-triggers/src/platforms/github.rs`
  - Config: `GitHubConfig` with token
  - Integration via webhooks for repository events

- **Jira** - Issue tracking and project management
  - Implementation: `JiraPlatform` in `crates/aof-triggers/src/platforms/jira.rs`
  - Config: `JiraConfig`

- **Microsoft Teams** - Enterprise team chat
  - Implementation: `TeamsPlatform` referenced in `aof-triggers`

- **GitLab** - Repository management and CI/CD
  - Implementation: `GitLabPlatform` in `crates/aof-triggers/src/platforms/gitlab.rs`

- **Bitbucket** - Repository management
  - Implementation: `BitbucketPlatform` in `crates/aof-triggers/src/platforms/bitbucket.rs`

- **OpsGenie** - Incident management
  - Implementation: `OpsGeniePlatform` in `crates/aof-triggers/src/platforms/opsgenie.rs`

- **PagerDuty** - On-call and incident response
  - Implementation: `PagerDutyPlatform` in `crates/aof-triggers/src/platforms/pagerduty.rs`
  - Config: `PagerDutyConfig`

**Infrastructure & Observability:**
- **Datadog** - Monitoring and observability
  - Tool implementation: `DatadogTool` in `crates/aof-tools/src/tools/datadog.rs`

- **Grafana** - Visualization and dashboards
  - Tool implementation: `GrafanaTool` in `crates/aof-tools/src/tools/grafana.rs`

- **New Relic** - APM and monitoring
  - Tool implementation: `NewRelicTool` in `crates/aof-tools/src/tools/newrelic.rs`

- **Splunk** - Log aggregation and analysis
  - Tool implementation: `SplunkTool` in `crates/aof-tools/src/tools/splunk.rs`

- **Prometheus** - Metrics collection
  - Referenced in observability tools

**DevOps/Cloud:**
- **Kubernetes** - Container orchestration
  - Tool implementation: `KubectlTool` in `crates/aof-tools/src/tools/kubectl.rs`
  - Direct CLI integration for cluster operations

- **Docker** - Container management
  - Tool implementation: `DockerTool` in `crates/aof-tools/src/tools/docker.rs`

- **Terraform** - Infrastructure as Code
  - Tool implementation: `TerraformTool` in `crates/aof-tools/src/tools/terraform.rs`

- **AWS** - Cloud services
  - Tool implementation: `AwsTool` in `crates/aof-tools/src/tools/aws.rs`
  - SDK: `aws-config`, `aws-sdk-bedrockruntime` for Bedrock

- **Google Cloud (GCP)** - Cloud services
  - Tool implementation: `GcpTool` in `crates/aof-tools/src/tools/gcp.rs`

- **Azure** - Cloud services
  - Tool implementation: `AzureTool` in `crates/aof-tools/src/tools/azure.rs`

- **HashiCorp Vault** - Secrets management
  - Tool implementation: `VaultTool` in `crates/aof-tools/src/tools/vault.rs`

**CI/CD Platforms:**
- **GitHub Actions** - CI/CD automation
  - Tool implementation: `GitHubActionsTool` in `crates/aof-tools/src/tools/github_actions.rs`

- **GitLab CI** - CI/CD pipelines
  - Tool implementation: `GitlabCiTool` in `crates/aof-tools/src/tools/gitlab_ci.rs`

- **ArgoCD** - GitOps CD tool
  - Tool implementation: `ArgoCdTool` in `crates/aof-tools/src/tools/argocd.rs`

- **Flux** - GitOps CD controller
  - Tool implementation: `FluxTool` in `crates/aof-tools/src/tools/flux.rs`

**Security & Compliance:**
- **Snyk** - Vulnerability scanning
  - Tool implementation: `SnykTool` in `crates/aof-tools/src/tools/snyk.rs`

- **Trivy** - Container and artifact scanning
  - Tool implementation: `TrivyTool` in `crates/aof-tools/src/tools/trivy.rs`

- **SonarQube** - Code quality analysis
  - Tool implementation: `SonarqubeTool` in `crates/aof-tools/src/tools/sonarqube.rs`

- **OPA/Conftest** - Policy as Code
  - Tool implementation: `OpaTool` in `crates/aof-tools/src/tools/opa.rs`

**ITSM:**
- **ServiceNow** - IT Service Management
  - Tool implementation: `ServiceNowTool` in `crates/aof-tools/src/tools/servicenow.rs`

**SIEM:**
- Generic SIEM tool implementations for security event correlation

## Data Storage

**Databases:**
- **Redis** (Optional Backend)
  - Client: `redis` crate 0.24 with tokio-comp and connection-manager
  - Connection: Configurable via backend initialization
  - Feature: `redis-backend` (optional)
  - Use: Distributed state caching (optional)

- **Sled** (Optional Backend)
  - Client: `sled` crate 0.34
  - Feature: `sled-backend` (optional)
  - Use: Embedded key-value store (optional)

**File Storage:**
- **Local Filesystem** (Default)
  - Backend: `FileBackend` in `aof-memory`
  - Location: Configurable (JSON file-based)
  - Persistence: Survives agent restarts

**In-Memory Storage:**
- **Default In-Memory Backend**
  - Implementation: `InMemoryBackend` in `aof-memory`
  - Storage: DashMap lock-free concurrent HashMap
  - Persistence: Ephemeral (cleared on restart)

## Caching

**Memory Caching:**
- **DashMap** - Lock-free concurrent HashMap for high-performance state access
  - Used throughout for agent state, tool results, activity tracking
  - No external caching service required by default

**Optional Distributed Caching:**
- **Redis** - Available via `redis-backend` feature

## Authentication & Identity

**LLM Provider Authentication:**
- **API Keys:**
  - `ANTHROPIC_API_KEY` - Anthropic Claude API
  - `OPENAI_API_KEY` - OpenAI GPT models
  - `GOOGLE_API_KEY` - Google Gemini
  - `GROQ_API_KEY` - Groq inference API
  - AWS credentials - Bedrock (via aws-config)

**Platform Webhook Authentication:**
- **Slack:** Signing secret verification (HMAC-SHA256)
  - Implementation: `verify_signature()` in `SlackPlatform`
  - Header: `X-Slack-Request-Timestamp`, `X-Slack-Signature`

- **GitHub:** Webhook signature verification (SHA-256)
  - Implementation: `verify_signature()` in `GitHubPlatform`

- **Discord:** Token-based authentication

- **Telegram:** Token-based authentication

- **Custom:** Cryptographic primitives available:
  - **hmac** 0.12 - HMAC signature generation/verification
  - **sha2** 0.10 - SHA-256 hashing
  - **ed25519-dalek** 2.1 - EdDSA signatures
  - **base64** 0.21 - Base64 encoding
  - **hex** 0.4 - Hex encoding

## Monitoring & Observability

**Error Tracking:**
- **ErrorKnowledgeBase** - In-core error pattern tracking
  - Location: `crates/aof-core/src/error_tracker.rs`
  - Purpose: Recurring error prevention and knowledge accumulation

**Logging:**
- **Tracing Framework** (0.1)
  - Structured logging with `tracing` crate
  - Log filtering via `tracing-subscriber` with `env-filter`
  - Integration point: All crates use `tracing::*` macros

**Observability Tools:**
- **Datadog, Grafana, New Relic, Splunk** - Via tool implementations

## CI/CD & Deployment

**Hosting:**
- **Docker** - Container-based deployment
  - Multi-stage Dockerfile provided
  - Base: Debian bookworm-slim
  - Build: Rust 1.75-slim-bookworm

**Build & Test:**
- `cargo build --release` - Release binary compilation
- `cargo test --lib` - Unit tests
- `./scripts/test-pre-compile.sh` - Fast pre-compile validation
- `./scripts/test-agent.sh` - End-to-end validation

**GitHub Actions:**
- Automated release workflow on version tag
- Binary builds for: Linux, macOS (Intel & Apple Silicon), Windows
- SHA256 checksum generation
- Automatic release notes generation

## Environment Configuration

**Required Environment Variables:**
- `ANTHROPIC_API_KEY` - For Anthropic Claude models
- `OPENAI_API_KEY` - For OpenAI GPT models
- `GOOGLE_API_KEY` - For Google Gemini models
- `GROQ_API_KEY` - For Groq models (optional)
- `OLLAMA_HOST` - For Ollama endpoint (defaults to `http://localhost:11434/v1`)

**AWS Credentials (for Bedrock):**
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_REGION`

**Platform Tokens:**
- `SLACK_BOT_TOKEN` - Slack bot authentication
- `SLACK_SIGNING_SECRET` - Slack webhook signature verification
- `DISCORD_BOT_TOKEN` - Discord bot token
- `TELEGRAM_BOT_TOKEN` - Telegram bot token
- `GITHUB_TOKEN` - GitHub API token
- Similar tokens for other platforms

**Configuration Files:**
- YAML-based configuration (parsed with `serde_yaml`)
- Precise error messages via `serde_path_to_error`
- No hardcoded secrets in codebase

## Webhooks & Callbacks

**Incoming Webhooks:**
- **Trigger Server** (`aof-triggers`)
  - Axum-based HTTP server with CORS support
  - Endpoints for each platform:
    - `/webhooks/slack` - Slack message and event handler
    - `/webhooks/discord` - Discord message handler
    - `/webhooks/telegram` - Telegram update handler
    - `/webhooks/github` - GitHub push and PR events
    - `/webhooks/jira` - Jira issue events
    - Similar endpoints for all supported platforms

**Webhook Features:**
- Signature verification per platform
- Rate limiting via `governor` (token bucket algorithm)
- Thread safety via `DashMap` concurrent storage
- Async request handling with Tokio

**Outgoing Callbacks:**
- **Platform Response Sending:**
  - Slack: `chat.postMessage`, `chat.scheduleMessage`
  - Discord: Direct message API
  - Telegram: `sendMessage`, `sendPhoto`
  - GitHub: `POST /repos/{owner}/{repo}/issues/{issue_number}/comments`
  - Similar patterns for all platforms

## Model Context Protocol (MCP)

**Transport Methods:**
- **Stdio** - Subprocess communication (default)
- **SSE** - Server-Sent Events (requires `reqwest`)
- **HTTP** - Direct HTTP calls (requires `reqwest`)

**Features:**
- Async client implementation in `aof-mcp`
- Request/response serialization via `serde_json`
- Tool calling protocol support
- Resource access patterns

## Cross-Platform Integration

**Platform Factory:**
- `PlatformFactory` and `PlatformRegistry` for extensible platform support
- `PlatformCapabilities` detection per platform
- `TypedPlatformConfig` for strongly-typed platform configuration
- Location: `crates/aof-triggers/src/platforms/mod.rs`

**Tool Framework:**
- Tool registry in `crates/aof-tools/src/registry.rs`
- 27+ tool implementations for various platforms and services
- Feature-gated tool compilation via cargo features

---

*Integration audit: 2026-02-11*
