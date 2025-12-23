---
sidebar_position: 2
---

# AWS Tools

AOF provides 11 AWS tools covering compute, storage, database, messaging, identity, and cost management.

## Prerequisites

- AWS CLI v2 installed
- Valid AWS credentials configured

```bash
# Install AWS CLI
curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip && sudo ./aws/install

# Configure credentials
aws configure
```

## Available Tools

| Tool | Service | Description |
|------|---------|-------------|
| `aws_s3` | S3 | Object storage operations |
| `aws_ec2` | EC2 | Compute instance management |
| `aws_logs` | CloudWatch Logs | Log querying and retrieval |
| `aws_iam` | IAM | Identity and access management |
| `aws_lambda` | Lambda | Serverless function operations |
| `aws_ecs` | ECS | Container service management |
| `aws_cloudformation` | CloudFormation | Infrastructure as code |
| `aws_rds` | RDS | Database service management |
| `aws_sqs` | SQS | Message queue operations |
| `aws_sns` | SNS | Notification service |
| `aws_cost` | Cost Explorer | Cost analysis and forecasting |

## Tool Reference

### aws_s3

S3 bucket and object operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `ls`, `cp`, `sync`, `rm`, `mb`, `rb`, `mv` |
| `source` | string | No | Source path (local or s3://) |
| `destination` | string | No | Destination path |
| `recursive` | boolean | No | Recursive operation |
| `region` | string | No | AWS region |
| `profile` | string | No | AWS profile name |

**Example:**

```yaml
tools:
  - aws_s3

# "List all buckets"
# "Copy backup.tar.gz to s3://my-bucket/backups/"
# "Sync local ./dist to s3://my-bucket/static/"
```

### aws_ec2

EC2 instance management.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `describe-instances`, `start-instances`, `stop-instances`, `reboot-instances`, `terminate-instances`, `describe-security-groups`, `describe-vpcs`, `describe-subnets` |
| `instance_ids` | array | No | Instance IDs |
| `filters` | array | No | Filters (e.g., `Name=tag:Env,Values=prod`) |
| `region` | string | No | AWS region |
| `profile` | string | No | AWS profile name |
| `output` | string | No | Output format: `json`, `text`, `table` |

**Example:**

```yaml
tools:
  - aws_ec2

# "List all running instances in us-east-1"
# "Stop instance i-1234567890abcdef0"
# "Find instances tagged with Environment=production"
```

### aws_logs

CloudWatch Logs operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `describe-log-groups`, `describe-log-streams`, `filter-log-events`, `get-log-events`, `tail` |
| `log_group_name` | string | No | Log group name |
| `log_stream_name` | string | No | Log stream name |
| `filter_pattern` | string | No | Filter pattern |
| `start_time` | string | No | Start time (Unix ms) |
| `end_time` | string | No | End time (Unix ms) |
| `limit` | integer | No | Max events (default: 100) |
| `region` | string | No | AWS region |
| `profile` | string | No | AWS profile name |

**Example:**

```yaml
tools:
  - aws_logs

# "Search /aws/lambda/my-function for errors in the last hour"
# "Tail logs from /ecs/my-service"
```

### aws_iam

IAM user, role, and policy management.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `list-users`, `list-roles`, `list-policies`, `get-user`, `get-role`, `get-policy`, `list-attached-role-policies`, `list-attached-user-policies` |
| `user_name` | string | No | IAM user name |
| `role_name` | string | No | IAM role name |
| `policy_arn` | string | No | Policy ARN |
| `profile` | string | No | AWS profile name |

**Example:**

```yaml
tools:
  - aws_iam

# "List all IAM users"
# "Get policies attached to role lambda-execution-role"
```

### aws_lambda

Lambda function operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `list-functions`, `get-function`, `invoke`, `list-versions-by-function`, `get-function-configuration` |
| `function_name` | string | No | Function name or ARN |
| `payload` | string | No | JSON payload for invoke |
| `region` | string | No | AWS region |
| `profile` | string | No | AWS profile name |

**Example:**

```yaml
tools:
  - aws_lambda

# "List all Lambda functions"
# "Invoke my-function with payload {\"key\": \"value\"}"
```

### aws_ecs

ECS cluster, service, and task management.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `list-clusters`, `list-services`, `list-tasks`, `describe-clusters`, `describe-services`, `describe-tasks`, `update-service`, `stop-task` |
| `cluster` | string | No | Cluster name or ARN |
| `service` | string | No | Service name |
| `tasks` | array | No | Task ARNs |
| `desired_count` | integer | No | Desired count for update |
| `region` | string | No | AWS region |
| `profile` | string | No | AWS profile name |

**Example:**

```yaml
tools:
  - aws_ecs

# "List all ECS clusters"
# "Scale my-service to 3 tasks in production cluster"
```

### aws_cloudformation

CloudFormation stack operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `describe-stacks`, `create-stack`, `delete-stack`, `list-stack-resources`, `describe-stack-events` |
| `stack_name` | string | No | Stack name |
| `template_body` | string | No | Template content |
| `template_url` | string | No | Template S3 URL |
| `parameters` | array | No | Stack parameters |
| `capabilities` | array | No | Required capabilities |
| `region` | string | No | AWS region |
| `profile` | string | No | AWS profile name |

**Example:**

```yaml
tools:
  - aws_cloudformation

# "Describe stack my-app-stack"
# "List resources in production-infrastructure stack"
```

### aws_rds

RDS database operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `describe-db-instances`, `describe-db-clusters`, `create-db-snapshot`, `describe-db-snapshots`, `stop-db-instance`, `start-db-instance` |
| `db_instance_identifier` | string | No | DB instance ID |
| `db_cluster_identifier` | string | No | DB cluster ID |
| `snapshot_identifier` | string | No | Snapshot ID |
| `region` | string | No | AWS region |
| `profile` | string | No | AWS profile name |

**Example:**

```yaml
tools:
  - aws_rds

# "List all RDS instances"
# "Create snapshot of production-db"
```

### aws_sqs

SQS queue operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `list-queues`, `send-message`, `receive-message`, `delete-message`, `get-queue-attributes`, `purge-queue` |
| `queue_url` | string | No | Queue URL |
| `message_body` | string | No | Message content |
| `receipt_handle` | string | No | Receipt handle for delete |
| `max_messages` | integer | No | Max messages to receive |
| `region` | string | No | AWS region |
| `profile` | string | No | AWS profile name |

**Example:**

```yaml
tools:
  - aws_sqs

# "List all SQS queues"
# "Send message to my-queue"
# "Receive 10 messages from processing-queue"
```

### aws_sns

SNS topic and subscription operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `list-topics`, `list-subscriptions`, `publish`, `create-topic`, `subscribe` |
| `topic_arn` | string | No | Topic ARN |
| `message` | string | No | Message to publish |
| `subject` | string | No | Message subject |
| `protocol` | string | No | Subscription protocol |
| `endpoint` | string | No | Subscription endpoint |
| `region` | string | No | AWS region |
| `profile` | string | No | AWS profile name |

**Example:**

```yaml
tools:
  - aws_sns

# "List all SNS topics"
# "Publish alert to alerts-topic"
```

### aws_cost

Cost Explorer operations for cost analysis.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `get-cost-and-usage`, `get-cost-forecast` |
| `time_period_start` | string | No | Start date (YYYY-MM-DD) |
| `time_period_end` | string | No | End date (YYYY-MM-DD) |
| `granularity` | string | No | `DAILY`, `MONTHLY` |
| `metrics` | array | No | Metrics to retrieve |
| `group_by` | string | No | Grouping dimension |
| `region` | string | No | AWS region |
| `profile` | string | No | AWS profile name |

**Example:**

```yaml
tools:
  - aws_cost

# "Get cost breakdown for the last 30 days grouped by service"
# "Forecast costs for the next month"
```

## Example Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: aws-ops
spec:
  model: google:gemini-2.5-flash
  tools:
    - aws_ec2
    - aws_s3
    - aws_logs
    - aws_cost

  environment:
    AWS_PROFILE: "production"
    AWS_DEFAULT_REGION: "us-east-1"

  system_prompt: |
    You are an AWS operations specialist.
    Help users manage EC2 instances, S3 storage, and analyze costs.
    Always confirm destructive operations before proceeding.
```

## Common Patterns

### Multi-Region Operations

```yaml
# Specify region per operation
"List EC2 instances in us-west-2"
"List EC2 instances in eu-west-1"
```

### Using Profiles

```yaml
environment:
  AWS_PROFILE: "production"

# Or specify per operation
"List S3 buckets using staging profile"
```

### Filtering Resources

```yaml
# EC2 filters
"Find instances where tag:Environment=production"
"Find instances where instance-state-name=running"
```
