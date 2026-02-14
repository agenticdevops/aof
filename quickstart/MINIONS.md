# 🦸 AOF Agent Minions - Quick Reference Card

## Your Complete Minion Squad (11 Agents - All Validated ✅)

### 🚀 Get Started in 30 Seconds

```bash
export GOOGLE_API_KEY="your-key-from-aistudio.google.com"
cd /Users/gshah/work/opsflow-sh/aof

# Try any minion:
aofctl run agent gojo/agents/kubo.yaml --prompt "Check K8s cluster health"
aofctl run agent gojo/agents/doku.yaml --prompt "Optimize my Dockerfile"
aofctl run agent gojo/agents/rafo.yaml --prompt "Design Terraform modules"
```

## Meet Your Minions

| Bot | Name | Specialty | Try It |
|-----|------|-----------|--------|
| 🐴 | **kubo** | Kubernetes (K8s expert) | `aofctl run agent gojo/agents/kubo.yaml --prompt "Check cluster health"` |
| 🐳 | **doku** | Docker (Containerization) | `aofctl run agent gojo/agents/doku.yaml --prompt "Optimize Dockerfile"` |
| 🏗️ | **rafo** | Terraform (IaC wizard) | `aofctl run agent gojo/agents/rafo.yaml --prompt "Design modules"` |
| ⚙️ | **ergo** | Argo (GitOps master) | `aofctl run agent gojo/agents/ergo.yaml --prompt "Argo Workflow DAG"` |
| ☁️ | **wos** | AWS (Cloud champion) | `aofctl run agent gojo/agents/wos.yaml --prompt "Design serverless"` |
| 🔵 | **zure** | Azure (Cloud specialist) | `aofctl run agent gojo/agents/zure.yaml --prompt "Azure architecture"` |
| 🐧 | **nux** | Linux (Sysadmin) | `aofctl run agent gojo/agents/nux.yaml --prompt "Why is it slow?"` |
| 📋 | **zibl** | Ansible (Orchestrator) | `aofctl run agent gojo/agents/zibl.yaml --prompt "Create playbook"` |

**Plus 3 utilities:**
- **quick-test** - General-purpose test agent
- **k8s-checker** - Kubernetes health diagnostics
- **system-monitor** - System resource monitoring

## Interactive Conversations

Have a multi-turn chat with any minion:

```bash
export GOOGLE_API_KEY="your-key-here"

# Example: Interactive Kubernetes troubleshooting
aofctl run agent gojo/agents/kubo.yaml --interactive

# Type questions, get responses, ask follow-ups
# Press Ctrl+C to exit
```

## Documentation

- **README.md** - Complete testing guide with scenarios
- **SETUP.md** - Quick start with copy-paste commands
- **AGENTS.md** - Detailed agent specifications and use cases
- **MINIONS.md** - This quick reference card

## Validation Status

✅ **All 11 agents validated and working**
✅ **Google Gemini 2.5 Flash configured**
✅ **YAML syntax correct (2-space indentation)**
✅ **Ready for production use**

## Architecture

```
gojo/
├── MINIONS.md              # ← You are here
├── SETUP.md                # Quick start guide
├── README.md               # Full documentation
├── AGENTS.md               # Detailed specifications
├── serve-config.yaml       # Server configuration
├── test-quick.sh           # Quick 5-minute test
├── test-interactive.sh     # Interactive test
└── agents/                 # Your minion squad
    ├── kubo.yaml          # K8s expert
    ├── doku.yaml          # Docker specialist
    ├── rafo.yaml          # Terraform wizard
    ├── ergo.yaml          # Argo orchestrator
    ├── wos.yaml           # AWS champion
    ├── zure.yaml          # Azure specialist
    ├── nux.yaml           # Linux admin
    ├── zibl.yaml          # Ansible master
    ├── quick-test.yaml    # General utility
    ├── k8s-checker.yaml   # K8s diagnostics
    └── system-monitor.yaml # System monitor
```

## Next Steps

1. **Test Your First Minion:**
   ```bash
   export GOOGLE_API_KEY="your-key-here"
   aofctl run agent gojo/agents/kubo.yaml --prompt "Hello! What can you do?"
   ```

2. **Run the Quick Test:**
   ```bash
   ./gojo/test-quick.sh
   ```

3. **Start the Full Stack:**
   - Terminal 1: `cargo run -p aofctl -- serve --config gojo/serve-config.yaml`
   - Terminal 2: `cd web-ui && npm run dev` (visit http://localhost:5173)
   - Terminal 3: `aofctl run agent gojo/agents/kubo.yaml --prompt "Hello!"`

4. **Use Different Models:**
   ```bash
   # Switch to Claude Sonnet
   aofctl run agent gojo/agents/kubo.yaml \
     --model "anthropic:claude-3-5-sonnet" \
     --prompt "Advanced Kubernetes question"
   ```

---

**You're ready to work with your minion squad! Pick one and start. 🚀**

For detailed information, see **AGENTS.md** for full specifications and use cases.
