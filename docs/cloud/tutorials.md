---
sidebar_position: 5
---

# Cloud Tutorials

Step-by-step tutorials for common multi-cloud workflows with AOF.

## Tutorial 1: Multi-Cloud Cost Analysis

Build an agent that analyzes costs across AWS, Azure, and GCP.

### Step 1: Create the Cost Analysis Agent

```yaml
# cost-analysis.yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: multi-cloud-cost-analyzer
  labels:
    category: finops
spec:
  model: google:gemini-2.5-flash
  max_tokens: 8192
  temperature: 0.2

  tools:
    - aws_cost
    - aws_ec2
    - azure_monitor
    - azure_vm
    - gcp_compute

  environment:
    AWS_PROFILE: "${AWS_PROFILE}"
    AZURE_SUBSCRIPTION_ID: "${AZURE_SUBSCRIPTION_ID}"
    GOOGLE_CLOUD_PROJECT: "${GOOGLE_CLOUD_PROJECT}"

  system_prompt: |
    You are a FinOps analyst specializing in multi-cloud cost optimization.

    ## Analysis Process
    1. Gather cost data from each cloud provider
    2. Identify top cost drivers per provider
    3. Find optimization opportunities
    4. Provide actionable recommendations

    ## Output Format
    ### Executive Summary
    - Total monthly spend: $X
    - Month-over-month change: +/-X%
    - Top optimization opportunity: [description]

    ### Cost Breakdown by Provider
    | Provider | Monthly Cost | % of Total | Trend |
    |----------|--------------|------------|-------|

    ### Top 5 Cost Drivers
    | Rank | Resource | Provider | Monthly Cost |
    |------|----------|----------|--------------|

    ### Optimization Recommendations
    | Priority | Action | Est. Savings | Effort |
    |----------|--------|--------------|--------|
```

### Step 2: Run the Analysis

```bash
# Analyze costs for the last 30 days
aofctl run agent cost-analysis.yaml \
  --input "Analyze cloud costs for the last 30 days across all providers"

# Focus on specific provider
aofctl run agent cost-analysis.yaml \
  --input "Deep dive into AWS EC2 costs and find rightsizing opportunities"

# Compare with previous period
aofctl run agent cost-analysis.yaml \
  --input "Compare this month's costs with last month and explain any increases"
```

---

## Tutorial 2: Cross-Cloud Resource Inventory

Build an agent that inventories resources across all cloud providers.

### Step 1: Create the Inventory Agent

```yaml
# cloud-inventory.yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: cloud-inventory
spec:
  model: google:gemini-2.5-flash
  max_tokens: 8192

  tools:
    # AWS
    - aws_ec2
    - aws_rds
    - aws_s3
    - aws_lambda
    - aws_ecs
    # Azure
    - azure_vm
    - azure_storage
    - azure_aks
    - azure_acr
    # GCP
    - gcp_compute
    - gcp_storage
    - gcp_gke
    - gcp_sql

  system_prompt: |
    You are a cloud infrastructure inventory specialist.

    ## Inventory Categories
    - Compute: VMs, containers, serverless
    - Storage: Object storage, block storage
    - Databases: Managed databases
    - Kubernetes: Clusters and node pools

    ## Output Format
    ### Resource Summary
    | Category | AWS | Azure | GCP | Total |
    |----------|-----|-------|-----|-------|
    | Compute  |     |       |     |       |
    | Storage  |     |       |     |       |
    | Database |     |       |     |       |
    | K8s      |     |       |     |       |

    ### Detailed Inventory
    [List resources by category with key attributes]
```

### Step 2: Run the Inventory

```bash
# Full inventory
aofctl run agent cloud-inventory.yaml \
  --input "Generate a complete inventory of all cloud resources"

# Category-specific inventory
aofctl run agent cloud-inventory.yaml \
  --input "List all Kubernetes clusters across AWS, Azure, and GCP"

# Export for CMDB
aofctl run agent cloud-inventory.yaml \
  --input "Generate inventory in JSON format for CMDB import" \
  --output-format json > inventory.json
```

---

## Tutorial 3: IAM Security Audit

Audit IAM configurations across cloud providers.

### Step 1: Create the Security Audit Agent

```yaml
# iam-audit.yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: iam-security-audit
spec:
  model: google:gemini-2.5-flash
  temperature: 0.1

  tools:
    - aws_iam
    - azure_keyvault
    - azure_resource
    - gcp_iam

  system_prompt: |
    You are an IAM security auditor. Check for:

    ## Security Checks
    1. **Over-permissioned accounts**
       - Admin access without justification
       - Wildcard (*) permissions

    2. **Credential hygiene**
       - Access keys older than 90 days
       - Unused credentials
       - Missing MFA

    3. **Service accounts**
       - Over-permissioned service accounts
       - Keys that should be rotated

    ## Risk Levels
    - CRITICAL: Immediate action required
    - HIGH: Address within 7 days
    - MEDIUM: Address within 30 days
    - LOW: Best practice recommendation

    ## Output Format
    ### Audit Summary
    | Provider | Users | Roles | Service Accounts | Critical | High |
    |----------|-------|-------|------------------|----------|------|

    ### Critical Findings
    | Provider | Resource | Issue | Remediation |
    |----------|----------|-------|-------------|

    ### Full Report
    [Detailed findings by category]
```

### Step 2: Run the Audit

```bash
# Full IAM audit
aofctl run agent iam-audit.yaml \
  --input "Perform a comprehensive IAM security audit"

# Focus on specific issue
aofctl run agent iam-audit.yaml \
  --input "Find all AWS IAM users without MFA enabled"

# Service account audit
aofctl run agent iam-audit.yaml \
  --input "Audit all service accounts for excessive permissions"
```

---

## Tutorial 4: Kubernetes Cluster Management

Manage Kubernetes clusters across EKS, AKS, and GKE.

### Step 1: Create the K8s Management Agent

```yaml
# k8s-manager.yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: multi-cloud-k8s
spec:
  model: google:gemini-2.5-flash

  tools:
    - aws_ecs  # For ECS/EKS context
    - azure_aks
    - gcp_gke
    - kubectl  # If kubectl is available

  system_prompt: |
    You are a multi-cloud Kubernetes specialist managing:
    - Amazon EKS
    - Azure AKS
    - Google GKE

    ## Capabilities
    - List and describe clusters
    - Get cluster credentials
    - Scale node pools
    - Check cluster health

    ## Best Practices
    - Always verify cluster before operations
    - Use node pool scaling for capacity changes
    - Document any configuration changes
```

### Step 2: Manage Clusters

```bash
# List all K8s clusters
aofctl run agent k8s-manager.yaml \
  --input "List all Kubernetes clusters across AWS, Azure, and GCP"

# Get credentials
aofctl run agent k8s-manager.yaml \
  --input "Get credentials for production AKS cluster"

# Scale operations
aofctl run agent k8s-manager.yaml \
  --input "Scale the default node pool in GKE cluster 'staging' to 5 nodes"
```

---

## Tutorial 5: Database Operations

Manage databases across RDS, Azure SQL, and Cloud SQL.

### Step 1: Create the Database Agent

```yaml
# db-manager.yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: multi-cloud-db
spec:
  model: google:gemini-2.5-flash
  temperature: 0.1

  tools:
    - aws_rds
    - gcp_sql
    - azure_monitor  # For Azure SQL metrics

  system_prompt: |
    You are a database administrator for multi-cloud environments.

    ## Responsibilities
    - List and describe database instances
    - Monitor database health
    - Manage backups and snapshots
    - Provide maintenance recommendations

    ## Safety Rules
    - NEVER delete production databases without explicit confirmation
    - Always verify backup completion before maintenance
    - Document all changes

    ## Database Comparison
    | Feature | AWS RDS | Azure SQL | Cloud SQL |
    |---------|---------|-----------|-----------|
    | Engine  | MySQL, PostgreSQL, etc. | SQL Server, PostgreSQL | MySQL, PostgreSQL |
    | HA      | Multi-AZ | Geo-replication | Regional |
```

### Step 2: Manage Databases

```bash
# List all databases
aofctl run agent db-manager.yaml \
  --input "List all database instances across cloud providers"

# Create backup
aofctl run agent db-manager.yaml \
  --input "Create a snapshot of RDS instance 'production-mysql'"

# Health check
aofctl run agent db-manager.yaml \
  --input "Check health and performance metrics for all production databases"
```

---

## Next Steps

- [Cloud Examples](/docs/cloud/examples) - Pre-built configurations
- [AWS Tools Reference](/docs/cloud/aws)
- [Azure Tools Reference](/docs/cloud/azure)
- [GCP Tools Reference](/docs/cloud/gcp)
