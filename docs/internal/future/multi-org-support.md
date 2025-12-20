# Multi-Organization Support

**GitHub Issue**: [#46](https://github.com/agenticdevops/aof/issues/46)
**Status**: Planned
**Priority**: High

## Summary

Enable native multi-organization support with per-org credentials, webhook secrets, and GitHub App installations for enterprise multi-tenant deployments.

## Current Limitation

All triggers must share the same `GITHUB_WEBHOOK_SECRET`:

```yaml
# triggers/org1.yaml
config:
  webhook_secret: ${GITHUB_WEBHOOK_SECRET}
  repositories: ["org1/*"]

# triggers/org2.yaml
config:
  webhook_secret: ${GITHUB_WEBHOOK_SECRET}  # Must be same
  repositories: ["org2/*"]
```

**Problems:**
- All GitHub orgs must configure the same webhook secret
- A single GitHub token must have access to all orgs
- No isolation between orgs
- Not suitable for MSP/consulting scenarios

## Proposed Solutions

### Option A: Per-Org Platform Configuration (Recommended)

```yaml
# daemon.yaml
platforms:
  github:
    organizations:
      - name: org1
        token_env: ORG1_GITHUB_TOKEN
        webhook_secret_env: ORG1_WEBHOOK_SECRET

      - name: org2
        token_env: ORG2_GITHUB_TOKEN
        webhook_secret_env: ORG2_WEBHOOK_SECRET

      - name: org3
        # GitHub App installation
        app_id_env: ORG3_APP_ID
        private_key_path_env: ORG3_PRIVATE_KEY_PATH
        installation_id_env: ORG3_INSTALLATION_ID
```

### Option B: Per-Trigger Credentials

```yaml
# triggers/org1.yaml
spec:
  type: GitHub
  config:
    # Override global credentials for this trigger
    token_env: ORG1_GITHUB_TOKEN
    webhook_secret_env: ORG1_WEBHOOK_SECRET
    repositories: ["org1/*"]
```

### Option C: GitHub App with Multi-Org Installations

```yaml
platforms:
  github:
    # Single GitHub App, multiple org installations
    app_id_env: GITHUB_APP_ID
    private_key_path_env: GITHUB_APP_PRIVATE_KEY

    installations:
      - org: org1
        installation_id_env: ORG1_INSTALLATION_ID
      - org: org2
        installation_id_env: ORG2_INSTALLATION_ID
```

## Implementation Plan

### Phase 1: Configuration Schema
- [ ] Extend DaemonConfig with `organizations` array
- [ ] Support per-org `token_env` and `webhook_secret_env`
- [ ] Validate configuration at startup

### Phase 2: Webhook Verification
- [ ] Extract org from webhook payload before signature verification
- [ ] Look up org-specific secret
- [ ] Verify signature with correct secret
- [ ] Fail fast if org not configured

### Phase 3: Token Selection
- [ ] Create org-aware token provider
- [ ] Select appropriate token for API calls
- [ ] Cache GitHub App installation tokens
- [ ] Handle token refresh for Apps

### Phase 4: Trigger Integration
- [ ] Pass org context to triggers
- [ ] Triggers can override org credentials
- [ ] Ensure API calls use org-specific tokens

## Webhook Verification Flow

```
Webhook Received
      │
      ▼
┌─────────────────┐
│ Parse payload   │
│ Extract org     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Look up org     │──── Not found ───▶ 404 Unknown Org
│ configuration   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Get org secret  │
│ from env        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Verify HMAC     │──── Invalid ───▶ 401 Unauthorized
│ signature       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Process event   │
│ with org token  │
└─────────────────┘
```

## Data Structures

```rust
pub struct OrganizationConfig {
    pub name: String,

    // PAT-based auth
    pub token_env: Option<String>,
    pub webhook_secret_env: String,

    // GitHub App auth
    pub app_id_env: Option<String>,
    pub private_key_path_env: Option<String>,
    pub installation_id_env: Option<String>,

    // Optional overrides
    pub api_url: Option<String>,  // For GHE
}

pub struct GitHubPlatformConfig {
    // Global defaults (optional)
    pub default_token_env: Option<String>,
    pub default_webhook_secret_env: Option<String>,

    // Per-org config
    pub organizations: Vec<OrganizationConfig>,
}
```

## Use Cases

1. **MSP/Consulting**: Manage multiple client GitHub orgs with isolated credentials
2. **Enterprise**: Different business units with separate GitHub orgs
3. **Hybrid**: Personal org + work org with different policies
4. **GitHub Enterprise**: Multiple GHE instances + github.com

## Security Considerations

- Secrets isolated per org
- No cross-org token access
- Audit logging per org
- Fail-closed if org not configured

## Related Files

- `crates/aofctl/src/commands/serve.rs` - Daemon configuration parsing
- `crates/aof-triggers/src/platforms/github.rs` - GitHub platform adapter
- `crates/aof-core/src/config.rs` - Configuration types

## Dependencies

- None (can be implemented independently)

## Acceptance Criteria

- [ ] Support per-org token configuration in DaemonConfig
- [ ] Support per-org webhook secret
- [ ] Support GitHub App with multiple installations
- [ ] Webhook signature verification works with per-org secrets
- [ ] API calls use org-specific credentials
- [ ] Triggers can override org credentials
- [ ] Documentation updated with multi-org examples
- [ ] Integration tests for multi-org scenarios
