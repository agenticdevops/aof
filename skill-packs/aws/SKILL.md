# AWS Operations Skill

You have expertise in AWS cloud infrastructure management. Apply these practices:

## CLI Usage
- Use `aws` CLI with `--output json` for structured output
- Always specify `--region` explicitly; never rely on defaults
- Use `--profile` for multi-account scenarios
- Prefer `--query` JMESPath expressions to filter large responses

## Safety Practices
- Before destructive operations (delete, terminate), list resources first
- Use `--dry-run` when available (EC2, S3 sync)
- Never use `--no-sign-request` in production contexts
- Verify resource names match exactly before modification

## Common Patterns
- EC2: `aws ec2 describe-instances --query 'Reservations[].Instances[].[InstanceId,State.Name,Tags[?Key==`Name`].Value[]]' --output table`
- S3: Always use `aws s3 ls s3://bucket --recursive --human-readable` before bulk operations
- IAM: Check existing policies before creating new ones; use least-privilege principles
- CloudWatch: Use `aws logs tail` for real-time log following
- Cost: Use `aws ce get-cost-and-usage` with daily granularity for cost analysis

## Error Handling
- `AccessDenied`: Check IAM permissions, confirm AWS credentials are configured
- `ThrottlingException`: Add exponential backoff; use `--max-attempts` flag
- `ResourceNotFoundException`: Verify region and resource name spelling
