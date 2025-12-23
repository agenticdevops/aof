---
sidebar_position: 6
---

# Cloud Examples

Practical examples for common cloud operations.

## Cost Management

### Analyze AWS Costs

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: aws-cost-analyzer
spec:
  model: google:gemini-2.5-flash
  tools:
    - aws_cost
    - aws_ec2

  system_prompt: |
    Analyze AWS costs and identify optimization opportunities.
    Focus on EC2 rightsizing and reserved instance recommendations.
```

**Usage:**

```bash
aofctl run agent aws-cost-analyzer.yaml \
  --input "Analyze costs for the last 30 days grouped by service"
```

**Expected Output:**

```
## Cost Analysis Summary
- Total Spend: $12,450
- Top Services: EC2 (45%), RDS (25%), S3 (15%)
- Month-over-Month: +8%

## Optimization Opportunities
| Resource | Current Cost | Recommended Action | Savings |
|----------|--------------|-------------------|---------|
| EC2 instances | $5,600 | Convert to Savings Plans | $1,120/mo |
| Unused EBS volumes | $340 | Delete 5 unattached volumes | $340/mo |
```

### Multi-Cloud Cost Report

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: multi-cloud-cost-report
spec:
  model: google:gemini-2.5-flash
  tools:
    - aws_cost
    - azure_monitor
    - gcp_compute

  environment:
    AWS_PROFILE: "${AWS_PROFILE}"
    AZURE_SUBSCRIPTION_ID: "${AZURE_SUBSCRIPTION_ID}"
    GOOGLE_CLOUD_PROJECT: "${GOOGLE_CLOUD_PROJECT}"

  system_prompt: |
    Generate a consolidated cost report across AWS, Azure, and GCP.
    Normalize costs and provide unified view.
```

---

## Security Operations

### IAM Audit Report

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: iam-audit-report
spec:
  model: google:gemini-2.5-flash
  tools:
    - aws_iam

  system_prompt: |
    Audit IAM for security issues:
    - Users without MFA
    - Overly permissive policies
    - Old access keys (>90 days)
    - Unused credentials
```

**Usage:**

```bash
aofctl run agent iam-audit-report.yaml \
  --input "Generate IAM security audit report"
```

### Service Account Audit

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: gcp-sa-audit
spec:
  model: google:gemini-2.5-flash
  tools:
    - gcp_iam

  system_prompt: |
    Audit GCP service accounts:
    - List all service accounts
    - Check key age and usage
    - Identify over-permissioned accounts
    - Recommend key rotation
```

---

## Infrastructure Operations

### EC2 Instance Management

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: ec2-manager
spec:
  model: google:gemini-2.5-flash
  tools:
    - aws_ec2

  system_prompt: |
    Manage EC2 instances. You can:
    - List instances with filters
    - Start/stop instances
    - Describe security groups
    - Find instances by tags
```

**Usage:**

```bash
# List running instances
aofctl run agent ec2-manager.yaml \
  --input "List all running EC2 instances in us-east-1"

# Find by tag
aofctl run agent ec2-manager.yaml \
  --input "Find all instances tagged with Environment=production"

# Stop instances
aofctl run agent ec2-manager.yaml \
  --input "Stop all instances in staging environment"
```

### Azure VM Operations

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: azure-vm-ops
spec:
  model: google:gemini-2.5-flash
  tools:
    - azure_vm
    - azure_monitor

  system_prompt: |
    Manage Azure VMs. You can:
    - List VMs by resource group
    - Start/stop/restart VMs
    - Check VM metrics
    - Monitor performance
```

**Usage:**

```bash
# List VMs
aofctl run agent azure-vm-ops.yaml \
  --input "List all VMs in resource group production-rg"

# Check performance
aofctl run agent azure-vm-ops.yaml \
  --input "Check CPU utilization for web-server VM"
```

### GCP Compute Management

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: gcp-compute-ops
spec:
  model: google:gemini-2.5-flash
  tools:
    - gcp_compute
    - gcp_logging

  system_prompt: |
    Manage GCP Compute Engine instances.
    - List instances by zone
    - Start/stop operations
    - Check logs for issues
```

---

## Kubernetes Operations

### Multi-Cloud K8s Dashboard

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: k8s-dashboard
spec:
  model: google:gemini-2.5-flash
  tools:
    - aws_ecs
    - azure_aks
    - gcp_gke

  system_prompt: |
    Provide a unified view of Kubernetes clusters across:
    - Amazon EKS
    - Azure AKS
    - Google GKE

    Report cluster health, node counts, and recent events.
```

**Usage:**

```bash
aofctl run agent k8s-dashboard.yaml \
  --input "Generate K8s cluster status report"
```

### AKS Cluster Operations

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: aks-ops
spec:
  model: google:gemini-2.5-flash
  tools:
    - azure_aks

  system_prompt: |
    Manage AKS clusters:
    - List clusters
    - Scale node pools
    - Get credentials
    - Check cluster health
```

---

## Database Operations

### RDS Management

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: rds-manager
spec:
  model: google:gemini-2.5-flash
  tools:
    - aws_rds

  system_prompt: |
    Manage RDS databases:
    - List instances and clusters
    - Create/restore snapshots
    - Start/stop instances
    - Check configuration
```

**Usage:**

```bash
# List databases
aofctl run agent rds-manager.yaml \
  --input "List all RDS instances"

# Create snapshot
aofctl run agent rds-manager.yaml \
  --input "Create snapshot of production-db before maintenance"
```

### Cloud SQL Operations

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: cloudsql-ops
spec:
  model: google:gemini-2.5-flash
  tools:
    - gcp_sql
    - gcp_logging

  system_prompt: |
    Manage Cloud SQL instances:
    - List instances
    - Manage backups
    - Check logs for issues
    - Restart if needed
```

---

## Storage Operations

### S3 Bucket Management

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: s3-manager
spec:
  model: google:gemini-2.5-flash
  tools:
    - aws_s3

  system_prompt: |
    Manage S3 buckets and objects:
    - List buckets and contents
    - Copy/sync files
    - Manage lifecycle
```

**Usage:**

```bash
# List buckets
aofctl run agent s3-manager.yaml \
  --input "List all S3 buckets with their sizes"

# Sync files
aofctl run agent s3-manager.yaml \
  --input "Sync ./dist folder to s3://my-bucket/assets/"
```

### Cross-Cloud Storage Sync

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: storage-sync
spec:
  model: google:gemini-2.5-flash
  tools:
    - aws_s3
    - azure_storage
    - gcp_storage

  system_prompt: |
    Help with cross-cloud storage operations.
    List contents and plan sync operations.
    NOTE: Actual cross-cloud sync requires external tools.
```

---

## Serverless Operations

### Lambda Management

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: lambda-ops
spec:
  model: google:gemini-2.5-flash
  tools:
    - aws_lambda
    - aws_logs

  system_prompt: |
    Manage Lambda functions:
    - List functions
    - Invoke functions
    - Check logs for errors
    - Review configuration
```

### Cloud Functions Management

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: gcf-ops
spec:
  model: google:gemini-2.5-flash
  tools:
    - gcp_functions
    - gcp_logging

  system_prompt: |
    Manage Cloud Functions:
    - List functions
    - Invoke with test data
    - Review execution logs
    - Check for errors
```

---

## Messaging Operations

### SQS Queue Management

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: sqs-manager
spec:
  model: google:gemini-2.5-flash
  tools:
    - aws_sqs

  system_prompt: |
    Manage SQS queues:
    - List queues
    - Check queue depth
    - Send/receive messages
    - Purge if needed
```

### Pub/Sub Management

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: pubsub-manager
spec:
  model: google:gemini-2.5-flash
  tools:
    - gcp_pubsub

  system_prompt: |
    Manage Pub/Sub topics and subscriptions:
    - List topics and subscriptions
    - Publish test messages
    - Check subscription backlog
```

---

## Fleet Example

### Complete Cloud Operations Fleet

```yaml
apiVersion: aof.sh/v1alpha1
kind: Fleet
metadata:
  name: cloud-ops-fleet
spec:
  agents:
    - name: cost-analyzer
      ref: library/cloud/cost-optimizer.yaml

    - name: security-auditor
      ref: library/cloud/iam-auditor.yaml

    - name: resource-manager
      ref: library/cloud/resource-rightsize.yaml

  workflow:
    - step: analyze-costs
      agent: cost-analyzer
      input: "Analyze costs for the last 30 days"

    - step: security-audit
      agent: security-auditor
      input: "Perform IAM security audit"
      parallel: true

    - step: rightsizing
      agent: resource-manager
      input: |
        Based on cost analysis, identify rightsizing opportunities.
        Cost data: {{ .steps.analyze-costs.output }}
```

**Usage:**

```bash
aofctl run fleet cloud-ops-fleet.yaml
```
