# AOF Agent Minions - The Complete Squad 🦸

Meet your specialized agent minions! Each bot is crafted with expertise in their domain and a unique personality.

## Quick Access

### Core Generalists
- **quick-test** - General-purpose test agent
- **general-assistant** - Multi-purpose helper

### Infrastructure & Platform Specialists
- **kubo** - Kubernetes expert (K8s orchestration)
- **doku** - Docker specialist (Containerization)
- **rafo** - Terraform wizard (Infrastructure-as-Code)
- **nux** - Linux administrator (System administration)
- **zibl** - Ansible orchestrator (Configuration management)
- **ergo** - Argo bot (GitOps & CI/CD)

### Cloud Platform Experts
- **wos** - AWS champion (Amazon Web Services)
- **zure** - Azure specialist (Microsoft cloud)

### Monitoring & Diagnostics
- **k8s-checker** - Kubernetes health checker
- **system-monitor** - System resource monitor

## Agent Specifications

### 🐴 Kubo - The Kubernetes Expert

```yaml
Name: kubo
Model: google:gemini-2.5-flash
Tools: kubectl, shell
Specialty: Kubernetes cluster management, troubleshooting, architecture
Personality: Helpful, precise, enthusiastic
```

**Use cases:**
```bash
aofctl run agent quickstart/agents/kubo.yaml --prompt "Check my cluster health"
aofctl run agent quickstart/agents/kubo.yaml --prompt "Why are my pods failing?"
aofctl run agent quickstart/agents/kubo.yaml --prompt "Design a K8s deployment strategy"
```

---

### 🐳 Doku - The Docker Expert

```yaml
Name: doku
Model: google:gemini-2.5-flash
Tools: shell
Specialty: Docker containerization, image optimization, registry management
Personality: Practical, detailed, passionate
```

**Use cases:**
```bash
aofctl run agent quickstart/agents/doku.yaml --prompt "Optimize my Dockerfile"
aofctl run agent quickstart/agents/doku.yaml --prompt "Create a Docker Compose setup"
aofctl run agent quickstart/agents/doku.yaml --prompt "Best practices for multi-stage builds"
```

---

### 🏗️ Rafo - The Terraform Wizard

```yaml
Name: rafo
Model: google:gemini-2.5-flash
Tools: shell
Specialty: Terraform modules, IaC best practices, state management
Personality: Methodical, detail-oriented, focused
```

**Use cases:**
```bash
aofctl run agent quickstart/agents/rafo.yaml --prompt "Design a multi-environment Terraform setup"
aofctl run agent quickstart/agents/rafo.yaml --prompt "Review my HCL for best practices"
aofctl run agent quickstart/agents/rafo.yaml --prompt "How should I structure modules?"
```

---

### ⚙️ Ergo - The GitOps Master

```yaml
Name: ergo
Model: google:gemini-2.5-flash
Tools: shell
Specialty: Argo Workflows, Argo CD, declarative deployments
Personality: Enthusiastic, process-focused, automation-dedicated
```

**Use cases:**
```bash
aofctl run agent quickstart/agents/ergo.yaml --prompt "Design a Argo Workflow DAG"
aofctl run agent quickstart/agents/ergo.yaml --prompt "Set up Argo CD for my cluster"
aofctl run agent quickstart/agents/ergo.yaml --prompt "How do I handle multi-cluster GitOps?"
```

---

### ☁️ Wos - The AWS Champion

```yaml
Name: wos
Model: google:gemini-2.5-flash
Tools: shell
Specialty: AWS services, cloud architecture, cost optimization
Personality: Knowledgeable, solution-oriented, optimization-focused
```

**Use cases:**
```bash
aofctl run agent quickstart/agents/wos.yaml --prompt "Design a serverless architecture on AWS"
aofctl run agent quickstart/agents/wos.yaml --prompt "How can I reduce my AWS costs?"
aofctl run agent quickstart/agents/wos.yaml --prompt "What IAM policies do I need?"
```

---

### 🔵 Zure - The Azure Specialist

```yaml
Name: zure
Model: google:gemini-2.5-flash
Tools: shell
Specialty: Azure services, enterprise solutions, hybrid cloud
Personality: Professional, enterprise-focused, integration-oriented
```

**Use cases:**
```bash
aofctl run agent quickstart/agents/zure.yaml --prompt "Design an enterprise Azure solution"
aofctl run agent quickstart/agents/zure.yaml --prompt "Set up Azure DevOps CI/CD"
aofctl run agent quickstart/agents/zure.yaml --prompt "Hybrid cloud with on-premises integration"
```

---

### 🐧 Nux - The Linux Administrator

```yaml
Name: nux
Model: google:gemini-2.5-flash
Tools: shell
Specialty: Linux system administration, shell scripting, OS optimization
Personality: Technical, thorough, Unix-principles focused
```

**Use cases:**
```bash
aofctl run agent quickstart/agents/nux.yaml --prompt "Why is my system slow?"
aofctl run agent quickstart/agents/nux.yaml --prompt "Write me an automation script"
aofctl run agent quickstart/agents/nux.yaml --prompt "How should I harden this server?"
```

---

### 📋 Zibl - The Ansible Orchestrator

```yaml
Name: zibl
Model: google:gemini-2.5-flash
Tools: shell
Specialty: Ansible playbooks, configuration management, automation
Personality: Organized, efficient, automation-obsessed
```

**Use cases:**
```bash
aofctl run agent quickstart/agents/zibl.yaml --prompt "Create an Ansible playbook for deployment"
aofctl run agent quickstart/agents/zibl.yaml --prompt "How do I structure complex playbooks?"
aofctl run agent quickstart/agents/zibl.yaml --prompt "Set up dynamic inventory management"
```

---

## Running Your Minions

### Single Prompt (Quick Task)
```bash
export GOOGLE_API_KEY="your-key-here"
aofctl run agent quickstart/agents/kubo.yaml --prompt "Check cluster health"
```

### Interactive Mode (Extended Conversation)
```bash
export GOOGLE_API_KEY="your-key-here"
aofctl run agent quickstart/agents/kubo.yaml
# Now have a multi-turn conversation with your minion
```

### Using Different Models
```bash
# Use your Anthropic subscription instead
aofctl run agent quickstart/agents/kubo.yaml \
  --model "anthropic:claude-3-5-sonnet" \
  --prompt "Advanced K8s architecture question"
```

## Minion Specialization Matrix

| Agent | Kubernetes | Docker | Cloud | Terraform | Automation | Linux |
|-------|:----------:|:------:|:-----:|:---------:|:----------:|:-----:|
| kubo  | ⭐⭐⭐ | - | - | - | - | - |
| doku  | ⭐ | ⭐⭐⭐ | - | - | - | ⭐ |
| rafo  | ⭐ | - | ⭐⭐ | ⭐⭐⭐ | ⭐ | ⭐ |
| ergo  | ⭐⭐ | - | - | - | ⭐⭐⭐ | ⭐ |
| wos   | ⭐ | ⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐ | ⭐ |
| zure  | ⭐ | ⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐ | ⭐ |
| nux   | ⭐ | - | - | - | ⭐⭐ | ⭐⭐⭐ |
| zibl  | ⭐ | ⭐ | - | - | ⭐⭐⭐ | ⭐⭐ |

⭐⭐⭐ = Expert  |  ⭐⭐ = Advanced  |  ⭐ = Knowledgeable

## Tips for Working with Minions

1. **Be Specific** - More detail = better recommendations
   ```bash
   # Good
   aofctl run agent quickstart/agents/kubo.yaml \
     --prompt "We have 3 nodes, running microservices, seeing 80% CPU usage. What's wrong?"
   
   # Less helpful
   aofctl run agent quickstart/agents/kubo.yaml --prompt "Help"
   ```

2. **Use Interactive Mode for Collaboration**
   ```bash
   aofctl run agent quickstart/agents/rafo.yaml
   # Follow-up questions, clarifications, iterative design
   ```

3. **Combine Minions for Complex Tasks**
   - First ask **Rafo** to design the infrastructure code
   - Then ask **Wos** to review the architecture
   - Then ask **Zibl** to create deployment automation
   - Finally ask **Nux** to set up monitoring

4. **Explore Before Committing**
   ```bash
   # Test locally first
   aofctl run agent quickstart/agents/kubo.yaml \
     --prompt "Dry run: what would this change do?"
   ```

---

**Ready to work with your minion squad? Pick one and get started! 🚀**
