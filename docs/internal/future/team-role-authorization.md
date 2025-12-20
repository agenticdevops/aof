# Team/Role-Based User Authorization

**GitHub Issue**: [#45](https://github.com/agenticdevops/aof/issues/45)
**Status**: Planned
**Priority**: High

## Summary

Extend `allowed_users` in GitHub triggers to support team membership, organization membership, and repository role-based authorization.

## Current Implementation

```yaml
config:
  allowed_users:
    - "alice"           # GitHub username
    - "bob"             # GitHub username
    - "dependabot[bot]" # Bot accounts
```

Simple string match against sender's login:

```rust
fn is_user_allowed(&self, username: &str) -> bool {
    if let Some(ref allowed) = self.config.allowed_users {
        allowed.contains(&username.to_string())
    } else {
        true
    }
}
```

## Proposed Enhancement

### New Syntax

```yaml
config:
  allowed_users:
    # Individual users (current behavior)
    - "alice"
    - "bob"

    # Team membership (new)
    - "team:myorg/sre-team"        # Members of GitHub team
    - "team:myorg/frontend-devs"

    # Organization membership (new)
    - "org:myorg"                  # Any member of the org

    # Repository role (new)
    - "role:admin"                 # Repo admins only
    - "role:maintainer"            # Maintainers and above
    - "role:write"                 # Write access and above
```

### API Calls Required

1. **Team membership**:
   ```
   GET /orgs/{org}/teams/{team_slug}/memberships/{username}
   ```

2. **Org membership**:
   ```
   GET /orgs/{org}/members/{username}
   ```

3. **Repository role**:
   ```
   GET /repos/{owner}/{repo}/collaborators/{username}/permission
   ```

## Implementation Plan

### Phase 1: Parser
- [ ] Parse `team:`, `org:`, `role:` prefixes
- [ ] Validate syntax at trigger load time
- [ ] Maintain backward compatibility with plain usernames

### Phase 2: API Integration
- [ ] Add GitHub API methods for membership checks
- [ ] Implement caching layer (TTL ~5 minutes)
- [ ] Handle rate limiting gracefully

### Phase 3: Authorization Logic
- [ ] Extend `is_user_allowed()` to handle all types
- [ ] Fail-closed on API errors (deny if can't verify)
- [ ] Log authorization decisions for audit

### Phase 4: Cross-Platform
- [ ] Apply same pattern to GitLab (`group:`, `project_role:`)
- [ ] Apply same pattern to Bitbucket (`workspace:`, `project_role:`)

## Caching Strategy

```rust
struct AuthCache {
    // Cache team membership: (org, team, user) -> bool
    team_cache: LruCache<(String, String, String), (bool, Instant)>,

    // Cache org membership: (org, user) -> bool
    org_cache: LruCache<(String, String), (bool, Instant)>,

    // Cache repo permission: (owner, repo, user) -> PermissionLevel
    role_cache: LruCache<(String, String, String), (PermissionLevel, Instant)>,

    ttl: Duration,  // Default: 5 minutes
}
```

## Use Cases

1. **SRE Team Only**: Allow only SRE team members to run `/deploy production`
2. **Org-Wide Access**: Allow any org member to run `/review`
3. **Maintainers Only**: Restrict `/approve` to repo maintainers
4. **Combined**: `team:myorg/sre OR role:admin` for critical operations

## Security Considerations

- Fail-closed: If API call fails, deny access
- Cache invalidation on team membership changes (webhook or short TTL)
- Audit logging for all authorization decisions
- Rate limit awareness (use conditional requests, ETags)

## Related Files

- `crates/aof-triggers/src/platforms/github.rs` - `is_user_allowed()`
- `crates/aof-triggers/src/platforms/gitlab.rs` - Similar implementation
- `crates/aof-triggers/src/platforms/bitbucket.rs` - Similar implementation

## Acceptance Criteria

- [ ] Support `team:org/team-name` syntax
- [ ] Support `org:org-name` syntax
- [ ] Support `role:admin|maintainer|write|read` syntax
- [ ] Cache API responses to avoid rate limits
- [ ] Update documentation with new syntax
- [ ] Add unit tests for authorization logic
- [ ] Add integration tests with mock GitHub API
- [ ] Apply same pattern to GitLab and Bitbucket
