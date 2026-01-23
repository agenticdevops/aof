---
sidebar_position: 13
sidebar_label: AWS
---

# AWS MCP Server

Interact with AWS services including EC2, S3, Lambda, CloudWatch, and more.

## Installation

```bash
# Using npx
npx -y @anthropic/mcp-server-aws

# Or via npm
npm install -g @anthropic/mcp-server-aws
```

## Configuration

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: aws-agent
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: aws
      command: npx
      args: ["-y", "@anthropic/mcp-server-aws"]
      env:
        AWS_ACCESS_KEY_ID: ${AWS_ACCESS_KEY_ID}
        AWS_SECRET_ACCESS_KEY: ${AWS_SECRET_ACCESS_KEY}
        AWS_REGION: us-east-1
```

### With IAM Role (EKS/EC2)

```yaml
mcp_servers:
  - name: aws
    command: npx
    args: ["-y", "@anthropic/mcp-server-aws"]
    env:
      AWS_REGION: us-east-1
      # Uses IAM role attached to pod/instance
```

### With Assumed Role

```yaml
mcp_servers:
  - name: aws
    command: npx
    args: ["-y", "@anthropic/mcp-server-aws"]
    env:
      AWS_REGION: us-east-1
      AWS_ROLE_ARN: arn:aws:iam::123456789:role/aof-agent-role
```

## Available Tools

### EC2

#### list_instances

List EC2 instances with filters.

```json
{
  "name": "list_instances",
  "arguments": {
    "filters": [
      {"Name": "tag:Environment", "Values": ["production"]},
      {"Name": "instance-state-name", "Values": ["running"]}
    ],
    "max_results": 50
  }
}
```

#### describe_instance

Get detailed instance information.

```json
{
  "name": "describe_instance",
  "arguments": {
    "instance_id": "i-1234567890abcdef0"
  }
}
```

#### get_instance_status

Get instance status checks.

```json
{
  "name": "get_instance_status",
  "arguments": {
    "instance_ids": ["i-1234567890abcdef0"]
  }
}
```

### S3

#### list_buckets

List S3 buckets.

```json
{
  "name": "list_buckets",
  "arguments": {}
}
```

#### list_objects

List objects in a bucket.

```json
{
  "name": "list_objects",
  "arguments": {
    "bucket": "my-bucket",
    "prefix": "logs/2024/",
    "max_keys": 100
  }
}
```

#### get_object

Get object content (text files only).

```json
{
  "name": "get_object",
  "arguments": {
    "bucket": "my-bucket",
    "key": "config/settings.json"
  }
}
```

### CloudWatch

#### get_metrics

Query CloudWatch metrics.

```json
{
  "name": "get_metrics",
  "arguments": {
    "namespace": "AWS/EC2",
    "metric_name": "CPUUtilization",
    "dimensions": [
      {"Name": "InstanceId", "Value": "i-1234567890abcdef0"}
    ],
    "start_time": "2024-01-15T00:00:00Z",
    "end_time": "2024-01-15T12:00:00Z",
    "period": 300,
    "statistic": "Average"
  }
}
```

#### get_alarms

List CloudWatch alarms.

```json
{
  "name": "get_alarms",
  "arguments": {
    "state_value": "ALARM",
    "alarm_name_prefix": "Production-"
  }
}
```

#### get_log_events

Get CloudWatch log events.

```json
{
  "name": "get_log_events",
  "arguments": {
    "log_group": "/aws/lambda/my-function",
    "log_stream": "2024/01/15/[$LATEST]abc123",
    "start_time": "2024-01-15T11:00:00Z",
    "end_time": "2024-01-15T12:00:00Z",
    "limit": 100
  }
}
```

### Lambda

#### list_functions

List Lambda functions.

```json
{
  "name": "list_functions",
  "arguments": {
    "max_items": 50
  }
}
```

#### get_function

Get function configuration.

```json
{
  "name": "get_function",
  "arguments": {
    "function_name": "my-function"
  }
}
```

#### invoke_function

Invoke a Lambda function.

```json
{
  "name": "invoke_function",
  "arguments": {
    "function_name": "my-function",
    "payload": {"key": "value"},
    "invocation_type": "RequestResponse"
  }
}
```

### Cost Explorer

#### get_cost_and_usage

Get cost and usage data.

```json
{
  "name": "get_cost_and_usage",
  "arguments": {
    "start": "2024-01-01",
    "end": "2024-01-31",
    "granularity": "DAILY",
    "metrics": ["UnblendedCost"],
    "group_by": [
      {"Type": "DIMENSION", "Key": "SERVICE"}
    ]
  }
}
```

## Use Cases

### Cost Optimizer Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: cost-optimizer
spec:
  model: google:gemini-2.5-flash
  instructions: |
    Analyze AWS costs and recommend optimizations.

    Check for:
    - Unused/idle EC2 instances
    - Unattached EBS volumes
    - Old snapshots
    - Right-sizing opportunities
    - Reserved instance recommendations
  mcp_servers:
    - name: aws
      command: npx
      args: ["-y", "@anthropic/mcp-server-aws"]
      env:
        AWS_REGION: us-east-1
```

### Infrastructure Monitor Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: infra-monitor
spec:
  model: google:gemini-2.5-flash
  instructions: |
    Monitor AWS infrastructure health.

    When alerted:
    1. Check CloudWatch alarms
    2. Get relevant metrics
    3. Check instance status
    4. Review recent logs
    5. Suggest remediation
  mcp_servers:
    - name: aws
      command: npx
      args: ["-y", "@anthropic/mcp-server-aws"]
      env:
        AWS_REGION: us-east-1
```

## Security Considerations

1. **IAM Policies**: Use least-privilege IAM policies
2. **Role Assumption**: Prefer IAM roles over static credentials
3. **Resource Tagging**: Restrict access by resource tags
4. **Audit Logging**: Enable CloudTrail for API auditing

### Example IAM Policy

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "ec2:Describe*",
        "cloudwatch:GetMetricData",
        "cloudwatch:DescribeAlarms",
        "logs:GetLogEvents",
        "s3:GetObject",
        "s3:ListBucket"
      ],
      "Resource": "*",
      "Condition": {
        "StringEquals": {
          "aws:ResourceTag/Environment": "production"
        }
      }
    }
  ]
}
```

## Troubleshooting

### Authentication Issues

```bash
# Verify credentials
aws sts get-caller-identity

# Test specific permission
aws ec2 describe-instances --dry-run
```

### Region Issues

```bash
# Check configured region
aws configure get region

# List available regions
aws ec2 describe-regions --output table
```

## Related

- [Cost Optimizer Agent](/docs/agent-library/cloud/cost-optimizer)
- [Capacity Planner Agent](/docs/agent-library/cloud/capacity-planner)
- [AWS Triggers](/docs/triggers/aws)
