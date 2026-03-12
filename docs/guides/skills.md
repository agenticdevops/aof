# Skills Composition

Skills are reusable capability modules that inject domain expertise into your agent's system prompt.
They are Markdown files (`SKILL.md`) — no code, no LLM routing call, just instructions.

## How Skills Work

When the gateway loads an agent directory, it reads every `skills/<name>/SKILL.md` file and
appends its content to the agent's system prompt under a `## Skill: <name>` heading.

Assembly order (deterministic — no LLM involved):
1. `SOUL.md` — base identity
2. `RULES.md` — constraints (if present)
3. `skills/<name>/SKILL.md` — one section per skill (alphabetical)

## Built-in Skill Packs

OpenAgentiX ships with 8 built-in skill packs embedded in the binary:

| Pack | Description |
|------|-------------|
| `aws` | AWS cloud infrastructure management |
| `kubernetes` | Kubernetes cluster management |
| `terraform` | Terraform infrastructure-as-code |
| `docker` | Docker container and image management |
| `git` | Git version control operations |
| `database` | Database operations and query optimization |
| `security` | Security scanning and hardening |
| `observability` | Monitoring, log analysis, distributed tracing |

List all packs:
```bash
agentix skills list
```

View a pack's content:
```bash
agentix skills show kubernetes
```

JSON output for scripting:
```bash
agentix skills list --output json
```

## Using a Built-in Skill Pack

To activate a built-in skill pack for your agent, create a directory with the pack name
under your agent's `skills/` directory:

```
agents/my-agent/
├── agent.yaml
├── SOUL.md
└── skills/
    └── kubernetes/     ← empty directory activates the built-in pack
```

An empty directory is sufficient — the built-in pack content is embedded in the binary.
The `DirectoryLoader` will use the built-in content when no custom `SKILL.md` is present.

## Creating a Custom Skill

Add a `SKILL.md` file inside a named skill directory:

```
agents/my-agent/
└── skills/
    └── postgres-tuning/
        └── SKILL.md    ← your custom instructions
```

Example `skills/postgres-tuning/SKILL.md`:
```markdown
# PostgreSQL Tuning Skill

Specialized knowledge for our production PostgreSQL cluster:

## Connection Details
- Production: Read from POSTGRES_HOST environment variable
- Always use read replicas for SELECT queries
- Connection pooling: PgBouncer on port 6432

## Tuning Queries
- Check slow queries: SELECT * FROM pg_stat_statements ORDER BY total_exec_time DESC LIMIT 10;
- Check autovacuum: SELECT * FROM pg_stat_user_tables WHERE n_dead_tup > 1000;
```

Custom skills with the same name as a built-in pack override the built-in content.
All other built-in packs remain available alongside your custom skills.

## Skill Loading at Runtime

The `DirectoryLoader` reads skills at agent load time (when `agentix gateway start` runs).
No LLM routing call is made — skill selection is entirely deterministic based on what
directories exist under `skills/`.

To reload agents after adding a skill:
- Restart the gateway, or
- Call `agentix apply -f agents/my-agent` (re-registers the agent)

## Multiple Skills

Combine multiple skills in one agent:

```
agents/k8s-ops/
├── agent.yaml
├── SOUL.md
└── skills/
    ├── kubernetes/         ← built-in Kubernetes pack
    ├── observability/      ← built-in observability pack
    └── our-cluster/
        └── SKILL.md        ← custom cluster-specific instructions
```

All three skill sections are injected into the system prompt, giving the agent
combined expertise across all declared domains.
