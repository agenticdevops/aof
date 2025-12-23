# Cloud Tools Design - Phase 5 (v0.6.0)

## Overview

Multi-cloud support for AWS, Azure, and GCP with comprehensive tool coverage and pre-built agents for common cloud operations.

## Architecture

### Tool Categories

```
cloud/
├── aws.rs       # Enhanced AWS tools (existing + new)
├── azure.rs     # Azure CLI tools (new)
└── gcp.rs       # GCP gcloud tools (new)
```

### Feature Flag

```toml
cloud = ["reqwest"]  # Optional HTTP for some operations
```

## AWS Tools (Enhanced)

### Existing Tools
| Tool | Service | Operations |
|------|---------|------------|
| `aws_s3` | S3 | ls, cp, sync, rm, mb, rb, mv |
| `aws_ec2` | EC2 | describe-instances, start, stop, reboot, terminate |
| `aws_logs` | CloudWatch | describe-log-groups, filter-events, tail |
| `aws_iam` | IAM | list-users, list-roles, get-user, get-role |
| `aws_lambda` | Lambda | list-functions, invoke, get-function |
| `aws_ecs` | ECS | list-clusters, list-services, describe-tasks |

### New Tools (Issue #68)
| Tool | Service | Operations |
|------|---------|------------|
| `aws_cloudformation` | CloudFormation | describe-stacks, create-stack, delete-stack, list-stack-resources |
| `aws_rds` | RDS | describe-db-instances, describe-db-clusters, create-db-snapshot |
| `aws_sqs` | SQS | list-queues, send-message, receive-message, get-queue-attributes |
| `aws_sns` | SNS | list-topics, list-subscriptions, publish |
| `aws_cost` | Cost Explorer | get-cost-and-usage, get-cost-forecast |

## Azure Tools (Issue #66)

### Tools
| Tool | Service | Operations |
|------|---------|------------|
| `azure_vm` | Compute | list, show, start, stop, restart, delete |
| `azure_storage` | Storage | account list, blob list, blob upload, blob download |
| `azure_aks` | AKS | list, show, get-credentials, scale |
| `azure_network` | Network | vnet list, nsg list, public-ip list |
| `azure_resource` | Resource Manager | group list, group show, resource list |
| `azure_keyvault` | Key Vault | secret list, secret show, secret set |
| `azure_monitor` | Monitor | metrics list, activity-log list, alert list |
| `azure_acr` | Container Registry | list, show, repository list |

### Prerequisites
- Azure CLI (`az`) installed
- Logged in via `az login`
- Subscription set via `az account set`

### Configuration
```bash
export AZURE_SUBSCRIPTION_ID="..."
export AZURE_TENANT_ID="..."
```

## GCP Tools (Issue #67)

### Tools
| Tool | Service | Operations |
|------|---------|------------|
| `gcp_compute` | Compute Engine | instances list, instances describe, instances start, instances stop |
| `gcp_storage` | Cloud Storage | ls, cp, rm, mb |
| `gcp_gke` | GKE | clusters list, clusters describe, clusters get-credentials |
| `gcp_iam` | IAM | roles list, service-accounts list, service-accounts keys list |
| `gcp_logging` | Cloud Logging | read, logs list |
| `gcp_pubsub` | Pub/Sub | topics list, subscriptions list, publish |
| `gcp_sql` | Cloud SQL | instances list, instances describe, backups list |
| `gcp_functions` | Cloud Functions | list, describe, call |

### Prerequisites
- Google Cloud SDK (`gcloud`) installed
- Authenticated via `gcloud auth login`
- Project set via `gcloud config set project`

### Configuration
```bash
export GOOGLE_CLOUD_PROJECT="..."
export GOOGLE_APPLICATION_CREDENTIALS="..."
```

## Pre-built Cloud Agents (Issue #69)

### cost-optimizer
```yaml
tools:
  - aws_cost
  - azure_monitor
  - gcp_compute
capabilities:
  - Analyze cloud spending across providers
  - Identify idle resources
  - Recommend reserved instances
  - Detect cost anomalies
```

### iam-auditor
```yaml
tools:
  - aws_iam
  - azure_resource
  - gcp_iam
capabilities:
  - Audit IAM policies and permissions
  - Detect over-permissioned roles
  - Identify unused credentials
  - Check for MFA compliance
```

### resource-rightsize
```yaml
tools:
  - aws_ec2
  - aws_rds
  - azure_vm
  - gcp_compute
capabilities:
  - Analyze resource utilization
  - Recommend instance type changes
  - Identify oversized databases
  - Suggest reserved capacity
```

### cloud-migrator
```yaml
tools:
  - aws_s3
  - azure_storage
  - gcp_storage
capabilities:
  - Plan cross-cloud migrations
  - Compare service equivalents
  - Estimate migration costs
  - Generate migration runbooks
```

## Implementation Pattern

All tools follow the CLI wrapper pattern:

```rust
pub struct AwsCloudFormationTool {
    config: ToolConfig,
}

impl AwsCloudFormationTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "command": {
                    "type": "string",
                    "enum": ["describe-stacks", "create-stack", "delete-stack", "list-stack-resources"]
                },
                "stack_name": { "type": "string" },
                "template_body": { "type": "string" },
                "region": { "type": "string" },
                "profile": { "type": "string" }
            }),
            vec!["command"],
        );

        Self {
            config: tool_config_with_timeout(
                "aws_cloudformation",
                "AWS CloudFormation stack operations",
                parameters,
                300,
            ),
        }
    }
}
```

## Error Handling

### Common Errors
| Error | Cause | Resolution |
|-------|-------|------------|
| CLI not found | AWS/Azure/GCP CLI not installed | Install CLI and add to PATH |
| Not authenticated | No valid credentials | Run login command |
| Permission denied | Insufficient IAM permissions | Grant required permissions |
| Region not set | No default region configured | Set region in config or parameter |

### Error Messages
Tools should return helpful error messages:
```json
{
  "success": false,
  "error": "Azure CLI not found. Install from https://docs.microsoft.com/cli/azure/install-azure-cli"
}
```

## Testing Strategy

### Unit Tests
- Mock CLI output for each command
- Test parameter validation
- Test error handling

### Integration Tests
- Use LocalStack for AWS
- Use Azure emulator or test subscription
- Use GCP emulator or test project

## Documentation Structure

```
docs/cloud/
├── overview.md         # Multi-cloud concepts
├── aws.md             # AWS tools reference
├── azure.md           # Azure tools reference
├── gcp.md             # GCP tools reference
├── tutorials.md       # Step-by-step guides
└── examples.md        # Agent configurations
```

## Success Criteria

1. **Tool Coverage**: 8+ operations per cloud provider
2. **Agent Templates**: 4 pre-built agents
3. **Documentation**: Complete reference + tutorials
4. **Testing**: All tools have unit tests
5. **Build**: Feature flag works independently
