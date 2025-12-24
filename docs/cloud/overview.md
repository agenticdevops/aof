---
sidebar_position: 1
---

# Cloud Provider Tools

AOF provides multi-cloud support for AWS, Azure, and GCP, enabling agents to manage resources across all major cloud platforms.

## Overview

Cloud tools wrap the native CLI tools (AWS CLI, Azure CLI, gcloud) to provide consistent, agent-friendly interfaces for cloud operations.

### Supported Providers

| Provider | CLI | Tools | Status |
|----------|-----|-------|--------|
| AWS | `aws` | 11 tools | Production |
| Azure | `az` | 8 tools | Production |
| GCP | `gcloud` | 8 tools | Production |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         AOF Agent                               │
├─────────────────────────────────────────────────────────────────┤
│                      Cloud Tools Layer                          │
├─────────────┬─────────────┬─────────────┬─────────────────────────┤
│   AWS CLI   │  Azure CLI  │   gcloud    │                         │
│   Wrapper   │   Wrapper   │   Wrapper   │                         │
├─────────────┼─────────────┼─────────────┤                         │
│  aws_s3     │  azure_vm   │ gcp_compute │     Unified API         │
│  aws_ec2    │  azure_aks  │ gcp_gke     │                         │
│  aws_rds    │  azure_*    │ gcp_*       │                         │
│  ...        │             │             │                         │
└─────────────┴─────────────┴─────────────┴─────────────────────────┘
```

## Quick Start

### Prerequisites

Install the CLI for your cloud provider:

```bash
# AWS CLI
curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip && sudo ./aws/install

# Azure CLI
curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash

# Google Cloud SDK
curl https://sdk.cloud.google.com | bash
```

### Authentication

```bash
# AWS
aws configure
# Or use environment variables:
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."

# Azure
az login
az account set --subscription "your-subscription-id"

# GCP
gcloud auth login
gcloud config set project your-project-id
```

### Using Cloud Tools

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: multi-cloud-ops
spec:
  model: google:gemini-2.5-flash
  tools:
    # AWS
    - aws_ec2
    - aws_s3
    - aws_cost
    # Azure
    - azure_vm
    - azure_storage
    # GCP
    - gcp_compute
    - gcp_storage

  system_prompt: |
    You manage resources across AWS, Azure, and GCP.
    Use the appropriate cloud tools based on the resource location.
```

## Tool Categories

### Compute

| AWS | Azure | GCP |
|-----|-------|-----|
| `aws_ec2` | `azure_vm` | `gcp_compute` |
| `aws_lambda` | `azure_functions` | `gcp_functions` |
| `aws_ecs` | `azure_aks` | `gcp_gke` |

### Storage

| AWS | Azure | GCP |
|-----|-------|-----|
| `aws_s3` | `azure_storage` | `gcp_storage` |

### Database

| AWS | Azure | GCP |
|-----|-------|-----|
| `aws_rds` | - | `gcp_sql` |

### Messaging

| AWS | Azure | GCP |
|-----|-------|-----|
| `aws_sqs` | - | `gcp_pubsub` |
| `aws_sns` | - | - |

### Identity & Security

| AWS | Azure | GCP |
|-----|-------|-----|
| `aws_iam` | `azure_keyvault` | `gcp_iam` |
| - | `azure_resource` | - |

### Monitoring & Logging

| AWS | Azure | GCP |
|-----|-------|-----|
| `aws_logs` | `azure_monitor` | `gcp_logging` |

### Infrastructure

| AWS | Azure | GCP |
|-----|-------|-----|
| `aws_cloudformation` | `azure_resource` | - |
| - | `azure_network` | - |

### Cost Management

| AWS | Azure | GCP |
|-----|-------|-----|
| `aws_cost` | `azure_monitor` | - |

## Pre-built Agents

AOF includes 4 pre-built agents for common cloud operations:

### cost-optimizer

Analyzes cloud spending and recommends optimizations.

```bash
aofctl run agent library/cloud/cost-optimizer.yaml \
  --input "Analyze AWS costs for the last 30 days"
```

### iam-auditor

Audits IAM policies and permissions for security issues.

```bash
aofctl run agent library/cloud/iam-auditor.yaml \
  --input "Audit IAM roles in production account"
```

### resource-rightsize

Analyzes resource utilization and recommends rightsizing.

```bash
aofctl run agent library/cloud/resource-rightsize.yaml \
  --input "Find oversized EC2 instances in us-east-1"
```

### cloud-migrator

Plans and assists with cross-cloud migrations.

```bash
aofctl run agent library/cloud/cloud-migrator.yaml \
  --input "Plan migration of S3 buckets to Azure Blob Storage"
```

## Feature Flag

Enable cloud tools in your build:

```toml
# Cargo.toml
aof-tools = { version = "0.2", features = ["cloud"] }
```

Or with all features:

```toml
aof-tools = { version = "0.2", features = ["all"] }
```

## Next Steps

- [AWS Tools Reference](/docs/cloud/aws)
- [Azure Tools Reference](/docs/cloud/azure)
- [GCP Tools Reference](/docs/cloud/gcp)
- [Cloud Tutorials](/docs/cloud/tutorials)
- [Cloud Examples](/docs/cloud/examples)
