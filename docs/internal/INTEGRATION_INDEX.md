# Platform Integration Documentation Index

**Purpose:** Central index for all platform integration documentation
**Platforms:** New Relic, Splunk, ServiceNow
**Status:** ✅ Architecture Analysis Complete

---

## 📁 Documentation Structure

```
docs/internal/
├── INTEGRATION_INDEX.md (this file)
├── INTEGRATION_ARCHITECTURE.md       ⭐ Complete architectural analysis
├── INTEGRATION_DIAGRAMS.md            ⭐ 14 Mermaid diagrams
├── INTEGRATION_API_SPEC.md            ⭐ Detailed API specifications
├── INTEGRATION_QUICK_REFERENCE.md     ⭐ Developer quick reference
└── ARCHITECTURE_ANALYSIS_SUMMARY.md   ⭐ Executive summary
```

---

## 🎯 Start Here

### For Architects & Product Managers
**Read first:** [ARCHITECTURE_ANALYSIS_SUMMARY.md](./ARCHITECTURE_ANALYSIS_SUMMARY.md)
- Executive summary with roadmap
- Architecture decision records (ADRs)
- Success metrics
- Estimated timelines (8-12 days total)

### For Developers (Implementation Team)
**Read first:** [INTEGRATION_QUICK_REFERENCE.md](./INTEGRATION_QUICK_REFERENCE.md)
- File templates
- Common patterns
- Checklists
- Testing templates
- Debugging tips

### For System Architects
**Read first:** [INTEGRATION_ARCHITECTURE.md](./INTEGRATION_ARCHITECTURE.md)
- Crate dependency graph
- Trait definitions
- Module structure
- Configuration patterns
- Architectural decisions

### For Visual Learners
**Read first:** [INTEGRATION_DIAGRAMS.md](./INTEGRATION_DIAGRAMS.md)
- 14 Mermaid diagrams
- Sequence diagrams
- Data flow diagrams
- Component relationships

### For API Integration Specialists
**Read first:** [INTEGRATION_API_SPEC.md](./INTEGRATION_API_SPEC.md)
- Complete API endpoint documentation
- Request/response formats
- Authentication patterns
- Error handling
- 15+ tool specifications

---

## 📋 Documents at a Glance

### 1. ARCHITECTURE_ANALYSIS_SUMMARY.md
**Length:** ~800 lines
**Content:**
- Executive summary
- Quick navigation
- Implementation roadmap (Phases 1-4)
- Key architectural patterns
- Architecture Decision Records (ADRs)
- Files to modify
- Testing strategy
- Security best practices
- Next steps

**Best for:** Project managers, architects, team leads

### 2. INTEGRATION_ARCHITECTURE.md
**Length:** ~1000 lines
**Content:**
- Crate dependency graph
- Core trait definitions (Tool, ToolExecutor)
- Reference implementation analysis (Datadog)
- Module structure recommendations
- Integration specifications (New Relic, Splunk, ServiceNow)
- Configuration schema patterns
- Implementation roadmap
- Architectural decisions (ADRs)
- Testing strategy
- Documentation requirements
- Key files to modify

**Best for:** System architects, senior developers

### 3. INTEGRATION_DIAGRAMS.md
**Length:** 14 diagrams
**Content:**
1. Crate dependency graph
2. Tool implementation layer
3. Tool trait implementation flow
4. Integration architecture layers
5. Tiered fleet execution (RCA example)
6. Tool registration flow
7. Authentication flow
8. Splunk async search pattern
9. Data flow: Incident response pipeline
10. Tool configuration schema
11. Feature flag dependencies
12. Error handling flow
13. Module organization
14. Deployment architecture

**Best for:** Visual learners, architects, onboarding

### 4. INTEGRATION_API_SPEC.md
**Length:** ~700 lines
**Content:**

**New Relic (5 tools):**
- newrelic_nrql_query
- newrelic_alert_list
- newrelic_entity_search
- newrelic_metric_data
- newrelic_incident_create

**Splunk (5 tools):**
- splunk_search (async pattern)
- splunk_saved_search_run
- splunk_alert_list
- splunk_event_submit
- splunk_index_list

**ServiceNow (6 tools):**
- servicenow_incident_create
- servicenow_incident_update
- servicenow_incident_query
- servicenow_cmdb_query
- servicenow_change_create
- servicenow_problem_create

**Common Patterns:**
- HTTP client configuration
- Error response handling
- Time parsing
- Async polling (Splunk)

**Best for:** Developers implementing tools, API specialists

### 5. INTEGRATION_QUICK_REFERENCE.md
**Length:** ~400 lines
**Content:**
- Implementation checklist
- File template (complete tool module)
- Common patterns (JSON schema, auth headers, error handling)
- Cargo.toml changes
- Module declarations
- Testing template
- Example YAML configuration
- Debugging tips
- Build & test commands
- Reference implementations

**Best for:** Developers during implementation, debugging

---

## 🗺️ Implementation Workflow

### Phase 1: New Relic (Days 1-3)
1. **Study:**
   - Read [INTEGRATION_API_SPEC.md](./INTEGRATION_API_SPEC.md) § New Relic section
   - Review Datadog reference implementation: `/Users/gshah/work/opsflow-sh/aof/crates/aof-tools/src/tools/datadog.rs`

2. **Implement:**
   - Use [INTEGRATION_QUICK_REFERENCE.md](./INTEGRATION_QUICK_REFERENCE.md) file template
   - Create `aof-tools/src/tools/newrelic.rs`
   - Implement 5 tools
   - Write unit tests

3. **Test:**
   - Follow checklist in [INTEGRATION_QUICK_REFERENCE.md](./INTEGRATION_QUICK_REFERENCE.md)
   - Create example YAML in `examples/observability/`

4. **Document:**
   - Create `docs/tools/newrelic.md`
   - Update `docs/DOCUMENTATION_INDEX.md`

### Phase 2: Splunk (Days 4-7)
1. **Study:**
   - Read [INTEGRATION_API_SPEC.md](./INTEGRATION_API_SPEC.md) § Splunk section
   - Review async polling pattern in [INTEGRATION_DIAGRAMS.md](./INTEGRATION_DIAGRAMS.md) (Diagram 8)

2. **Implement:**
   - Create `aof-tools/src/tools/splunk.rs`
   - Implement async search pattern (create job → poll → results)
   - Implement 5 tools
   - Write unit tests

3. **Test:**
   - Test polling with mock delays
   - Test timeout handling
   - Create example YAML

4. **Document:**
   - Create `docs/tools/splunk.md`
   - Document async pattern

### Phase 3: ServiceNow (Days 8-11)
1. **Study:**
   - Read [INTEGRATION_API_SPEC.md](./INTEGRATION_API_SPEC.md) § ServiceNow section
   - Study encoded query syntax

2. **Implement:**
   - Create `aof-tools/src/tools/servicenow.rs`
   - Implement encoded query builder
   - Implement 6 tools
   - Write unit tests

3. **Test:**
   - Test CRUD operations
   - Test query encoding
   - Create example YAML

4. **Document:**
   - Create `docs/tools/servicenow.md`
   - Document encoded queries

### Phase 4: Integration (Days 12)
1. **Create fleet example:**
   - Incident response pipeline (New Relic + Splunk + ServiceNow)
   - Multi-tier execution (data collectors → reasoners → coordinator)

2. **Test end-to-end:**
   - Run complete workflow
   - Verify tier coordination
   - Validate incident creation

3. **Document:**
   - Create `docs/tutorials/incident-response.md`
   - Update `docs/getting-started.md`

---

## 🔧 Tools to Implement (16 Total)

### New Relic (5 tools)
- [x] ✅ Documented in API spec
- [x] ✅ Example YAML defined
- [ ] 🔨 Implementation pending

1. `newrelic_nrql_query` - NRQL queries
2. `newrelic_alert_list` - Alert violations
3. `newrelic_entity_search` - Entity search
4. `newrelic_metric_data` - Timeseries metrics
5. `newrelic_incident_create` - Create incidents

### Splunk (5 tools)
- [x] ✅ Documented in API spec
- [x] ✅ Async pattern defined
- [ ] 🔨 Implementation pending

1. `splunk_search` - SPL search (async)
2. `splunk_saved_search_run` - Run saved search
3. `splunk_alert_list` - Triggered alerts
4. `splunk_event_submit` - HEC event submission
5. `splunk_index_list` - List indexes

### ServiceNow (6 tools)
- [x] ✅ Documented in API spec
- [x] ✅ Encoded query syntax defined
- [ ] 🔨 Implementation pending

1. `servicenow_incident_create` - Create incidents
2. `servicenow_incident_update` - Update incidents
3. `servicenow_incident_query` - Query incidents
4. `servicenow_cmdb_query` - CMDB queries
5. `servicenow_change_create` - Change requests
6. `servicenow_problem_create` - Problem tickets

---

## 📊 Architecture at a Glance

### Crate Structure
```
aof-core (traits) → aof-tools (implementations) → aof-runtime (execution) → aofctl (CLI)
```

### Tool Trait
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult>;
    fn config(&self) -> &ToolConfig;
}
```

### Module Organization
```
aof-tools/src/tools/
├── mod.rs (declarations)
├── common.rs (helpers)
├── datadog.rs (reference)
├── newrelic.rs (NEW)
├── splunk.rs (NEW)
└── servicenow.rs (NEW)
```

### Feature Flags
```toml
[features]
observability = ["reqwest", "chrono"]  # New Relic
siem = ["reqwest", "chrono"]           # Splunk
itsm = ["reqwest", "chrono", "base64"] # ServiceNow
```

---

## 🎓 Learning Resources

### Code References
- **Datadog (HTTP API):** `/Users/gshah/work/opsflow-sh/aof/crates/aof-tools/src/tools/datadog.rs`
- **Grafana (REST API):** `/Users/gshah/work/opsflow-sh/aof/crates/aof-tools/src/tools/grafana.rs`
- **Observability tools:** `/Users/gshah/work/opsflow-sh/aof/crates/aof-tools/src/tools/observability.rs`
- **Tool registry:** `/Users/gshah/work/opsflow-sh/aof/crates/aof-tools/src/registry.rs`
- **Agent config:** `/Users/gshah/work/opsflow-sh/aof/crates/aof-core/src/agent.rs`
- **Fleet config:** `/Users/gshah/work/opsflow-sh/aof/crates/aof-core/src/fleet.rs`

### External API Documentation
- **New Relic API:** https://docs.newrelic.com/docs/apis/rest-api-v2/
- **Splunk REST API:** https://docs.splunk.com/Documentation/Splunk/latest/RESTREF/RESTprolog
- **ServiceNow Table API:** https://docs.servicenow.com/bundle/tokyo-application-development/page/integrate/inbound-rest/concept/c_TableAPI.html

### Internal Documentation
- **AOF Architecture:** `/Users/gshah/work/opsflow-sh/aof/docs/architecture/`
- **Tool documentation:** `/Users/gshah/work/opsflow-sh/aof/docs/tools/`
- **Example agents:** `/Users/gshah/work/opsflow-sh/aof/examples/`
- **Getting started:** `/Users/gshah/work/opsflow-sh/aof/docs/getting-started.md`

---

## ✅ Deliverables Checklist

### Code
- [ ] `aof-tools/src/tools/newrelic.rs` (5 tools)
- [ ] `aof-tools/src/tools/splunk.rs` (5 tools)
- [ ] `aof-tools/src/tools/servicenow.rs` (6 tools)
- [ ] Unit tests (mockito-based)
- [ ] Integration tests (optional, real API)
- [ ] Update `Cargo.toml` (feature flags: siem, itsm)
- [ ] Update `mod.rs` (module declarations)
- [ ] Update `lib.rs` (exports)

### Examples
- [ ] `examples/observability/newrelic-agent.yaml`
- [ ] `examples/observability/splunk-investigation.yaml`
- [ ] `examples/itsm/servicenow-automation.yaml`
- [ ] `examples/fleets/incident-response-pipeline.yaml`

### Documentation
- [ ] `docs/tools/newrelic.md` (user guide)
- [ ] `docs/tools/splunk.md` (user guide)
- [ ] `docs/tools/servicenow.md` (user guide)
- [ ] `docs/tutorials/incident-response.md` (end-to-end tutorial)
- [ ] Update `docs/getting-started.md` (add integration examples)
- [ ] Update `docs/DOCUMENTATION_INDEX.md` (add new docs)

### Testing
- [ ] All unit tests pass (`cargo test --lib --features all`)
- [ ] No clippy warnings (`cargo clippy --features all`)
- [ ] Formatted code (`cargo fmt`)
- [ ] Example YAMLs validated (`aofctl validate`)
- [ ] End-to-end fleet test

---

## 🚀 Quick Start Commands

```bash
# Navigate to project
cd /Users/gshah/work/opsflow-sh/aof

# Read architecture docs
cat docs/internal/ARCHITECTURE_ANALYSIS_SUMMARY.md
cat docs/internal/INTEGRATION_QUICK_REFERENCE.md

# Study reference implementation
cat crates/aof-tools/src/tools/datadog.rs

# Create feature branch
git checkout -b feature/platform-integrations

# Create tool module (use template from quick reference)
touch crates/aof-tools/src/tools/newrelic.rs

# Check syntax
cargo check --features observability

# Run tests
cargo test --lib --features observability

# Build
cargo build --release --features observability,siem,itsm
```

---

## 📞 Support & Questions

### For Architecture Questions
- **Reference:** [INTEGRATION_ARCHITECTURE.md](./INTEGRATION_ARCHITECTURE.md)
- **Diagrams:** [INTEGRATION_DIAGRAMS.md](./INTEGRATION_DIAGRAMS.md)
- **ADRs:** [ARCHITECTURE_ANALYSIS_SUMMARY.md](./ARCHITECTURE_ANALYSIS_SUMMARY.md) § Architecture Decision Records

### For Implementation Questions
- **Reference:** [INTEGRATION_QUICK_REFERENCE.md](./INTEGRATION_QUICK_REFERENCE.md)
- **API Specs:** [INTEGRATION_API_SPEC.md](./INTEGRATION_API_SPEC.md)
- **Code Example:** `/Users/gshah/work/opsflow-sh/aof/crates/aof-tools/src/tools/datadog.rs`

### For Testing Questions
- **Templates:** [INTEGRATION_QUICK_REFERENCE.md](./INTEGRATION_QUICK_REFERENCE.md) § Testing Template
- **Strategy:** [INTEGRATION_ARCHITECTURE.md](./INTEGRATION_ARCHITECTURE.md) § Testing Strategy

---

## 📈 Progress Tracking

| Phase | Platform | Tools | Status | Est. Days |
|-------|----------|-------|--------|-----------|
| 1 | New Relic | 5 | 📋 Planned | 2-3 |
| 2 | Splunk | 5 | 📋 Planned | 3-4 |
| 3 | ServiceNow | 6 | 📋 Planned | 3-4 |
| 4 | Integration | Fleet example | 📋 Planned | 1-2 |
| **Total** | **3 platforms** | **16 tools** | **📋 Planned** | **8-12** |

---

## 🎯 Success Criteria

### Code Quality
- ✅ All tools follow Datadog reference pattern
- ✅ 100% unit test coverage (mocked HTTP)
- ✅ Comprehensive error handling (401, 403, 404, 429, 5xx)
- ✅ No clippy warnings
- ✅ Helpful error messages with context

### Documentation
- ✅ Inline Rust doc comments for all public types
- ✅ User-facing docs for each platform
- ✅ Example YAML configurations
- ✅ Troubleshooting guides

### User Experience
- ✅ Consistent YAML configuration syntax
- ✅ Clear validation errors
- ✅ Credentials from Context (not hardcoded)
- ✅ Multi-platform fleet examples

---

## 📝 Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-12-25 | Architect Agent | Initial architecture analysis complete |

---

**Status:** ✅ Analysis Complete, Ready for Implementation
**Next Step:** Begin Phase 1 (New Relic Integration)
**Estimated Completion:** 8-12 days from start
