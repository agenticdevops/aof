---
sidebar_position: 7
sidebar_label: Cloud
---

# Cloud Agents

Production-ready agents for cloud operations, cost optimization, and infrastructure management.

## Overview

| Agent | Purpose | Tools |
|-------|---------|-------|
| [cost-optimizer](#cost-optimizer) | Optimize cloud costs | aws_*, gcp_*, kubectl |
| [iam-auditor](#iam-auditor) | Audit IAM permissions | aws_iam, gcp_iam, kubectl |
| [resource-rightsizer](#resource-rightsizer) | Right-size resources | aws_*, gcp_*, kubectl |
| [cloud-migrator](#cloud-migrator) | Migration planning | Multiple |
| [drift-detector](#drift-detector) | Detect infrastructure drift | terraform, kubectl |

## cost-optimizer

Analyzes cloud spending and recommends cost optimizations.

### Usage

```bash
# Analyze AWS costs
aofctl run agent library://cloud/cost-optimizer \
  --prompt "Analyze our AWS spending and find savings opportunities"

# Identify waste
aofctl run agent library://cloud/cost-optimizer \
  --prompt "Find unused or underutilized resources in GCP"
```

### Capabilities

- Cost breakdown by service/team/tag
- Unused resource detection
- Reserved instance recommendations
- Spot instance opportunities
- Storage optimization
- Network cost analysis

### Tools Used
- `aws_ce` - AWS Cost Explorer
- `aws_ec2` - EC2 analysis
- `aws_rds` - RDS analysis
- `gcp_billing` - GCP billing
- `kubectl` - Kubernetes workloads

### Example Output

```markdown
## Cloud Cost Optimization Report

### Monthly Spend Overview

| Cloud | Current | Last Month | Change |
|-------|---------|------------|--------|
| AWS | $45,230 | $42,100 | +7.4% |
| GCP | $12,450 | $11,800 | +5.5% |
| **Total** | **$57,680** | **$53,900** | **+7.0%** |

### Cost by Category

```
AWS ($45,230)
├── EC2 Compute: $18,500 (41%)
├── RDS Database: $12,300 (27%)
├── S3 Storage: $5,200 (11%)
├── Data Transfer: $4,800 (11%)
└── Other: $4,430 (10%)
```

### Savings Opportunities

#### 1. Idle EC2 Instances ($3,200/month)
| Instance | Type | Avg CPU | Running Since | Monthly Cost |
|----------|------|---------|---------------|--------------|
| dev-server-01 | m5.xlarge | 2% | 180 days | $140 |
| test-env-03 | r5.2xlarge | 0% | 90 days | $520 |
| legacy-api | c5.4xlarge | 5% | 365 days | $980 |

**Recommendation**: Terminate or schedule auto-stop

#### 2. Reserved Instance Opportunities ($8,500/month)
| Service | On-Demand | 1-Year RI | 3-Year RI | Savings |
|---------|-----------|-----------|-----------|---------|
| EC2 Compute | $18,500 | $12,900 | $9,200 | 30-50% |
| RDS | $12,300 | $8,600 | $6,100 | 30-50% |

**Recommendation**: Purchase RIs for stable workloads

#### 3. Oversized Resources ($2,800/month)
| Resource | Current | Recommended | Savings |
|----------|---------|-------------|---------|
| api-server | m5.2xlarge | m5.large | $380/mo |
| cache | r5.xlarge | r5.large | $180/mo |
| db-primary | db.r5.4xl | db.r5.2xl | $1,200/mo |

**Recommendation**: Rightsize based on utilization

#### 4. Storage Optimization ($1,500/month)
| Finding | Current Cost | Savings |
|---------|--------------|---------|
| Unattached EBS volumes | $450 | $450 |
| Old S3 objects (>90 days) | $2,100 | $600 (lifecycle) |
| Snapshot retention | $1,800 | $450 (cleanup) |

### Total Potential Savings

| Category | Monthly Savings | Annual Savings |
|----------|-----------------|----------------|
| Idle Resources | $3,200 | $38,400 |
| Reserved Instances | $8,500 | $102,000 |
| Rightsizing | $2,800 | $33,600 |
| Storage | $1,500 | $18,000 |
| **Total** | **$16,000** | **$192,000** |

### Implementation Priority

1. **Immediate** (This week): Terminate idle instances
2. **Short-term** (This month): Purchase RIs for stable workloads
3. **Medium-term** (Next quarter): Implement rightsizing
```

---

## iam-auditor

Audits IAM permissions and access controls across cloud providers.

### Usage

```bash
# Audit AWS IAM
aofctl run agent library://cloud/iam-auditor \
  --prompt "Audit IAM policies for overly permissive access"

# Review service accounts
aofctl run agent library://cloud/iam-auditor \
  --prompt "Review Kubernetes service account permissions"
```

### Capabilities

- Overly permissive policy detection
- Unused credentials identification
- Cross-account access review
- Service account auditing
- Privilege escalation paths
- Compliance checking

### Tools Used
- `aws_iam` - AWS IAM
- `gcp_iam` - GCP IAM
- `kubectl` - Kubernetes RBAC

### Example Output

```markdown
## IAM Security Audit Report

### Summary

| Provider | Users | Roles | Issues Found |
|----------|-------|-------|--------------|
| AWS | 45 | 32 | 18 |
| GCP | 28 | 15 | 8 |
| K8s | - | 67 | 12 |

### Critical Findings

#### 1. Admin Access Too Broad (AWS)
**Finding**: 8 users have full AdministratorAccess

| User | Last Active | MFA | Risk |
|------|-------------|-----|------|
| dev-user-01 | 90 days ago | No | CRITICAL |
| contractor-a | 180 days ago | No | CRITICAL |
| service-account | N/A | N/A | HIGH |

**Recommendation**:
- Remove admin access from inactive users
- Implement least privilege policies
- Enable MFA for all admin users

#### 2. Wildcard Permissions (AWS)
**Finding**: 12 policies with `*` resource permissions

```json
{
  "Effect": "Allow",
  "Action": "s3:*",
  "Resource": "*"  // ← Too permissive
}
```

**Affected Policies**:
- DataTeamPolicy (3 users)
- LegacyAppPolicy (5 roles)
- DevPolicy (12 users)

#### 3. Unused Credentials
| Credential | Type | Last Used | Age |
|------------|------|-----------|-----|
| AKIA...XYZ | Access Key | Never | 180 days |
| AKIA...ABC | Access Key | 90 days ago | 365 days |
| sa-backup@ | Service Account | 120 days ago | 200 days |

### Kubernetes RBAC Issues

#### Cluster-Admin Bindings
| Subject | Type | Namespace | Risk |
|---------|------|-----------|------|
| dev-team | Group | cluster-wide | HIGH |
| debug-sa | ServiceAccount | default | MEDIUM |

#### Overly Permissive Roles
| Role | Permissions | Used By | Risk |
|------|-------------|---------|------|
| full-access | */resources | 3 SAs | HIGH |
| pod-admin | pods/* | 5 SAs | MEDIUM |

### Remediation Plan

**Priority 1** (This Week):
1. Remove admin access from inactive users
2. Delete unused access keys
3. Enable MFA for remaining admins

**Priority 2** (This Month):
1. Replace wildcard policies with scoped permissions
2. Review and reduce cluster-admin bindings
3. Implement automated credential rotation

**Priority 3** (This Quarter):
1. Implement AWS Organizations SCPs
2. Set up IAM Access Analyzer
3. Automated compliance monitoring
```

---

## resource-rightsizer

Analyzes resource utilization and recommends right-sizing.

### Usage

```bash
# Analyze EC2 instances
aofctl run agent library://cloud/resource-rightsizer \
  --prompt "Analyze EC2 instance utilization and recommend sizes"

# Kubernetes workloads
aofctl run agent library://cloud/resource-rightsizer \
  --prompt "Right-size Kubernetes deployments in production"
```

### Capabilities

- CPU/Memory utilization analysis
- Instance type recommendations
- Cost-performance optimization
- Reserved capacity planning
- Scaling recommendations
- Historical trend analysis

### Tools Used
- `aws_cloudwatch` - Metrics
- `aws_ec2` - Instance details
- `gcp_monitoring` - GCP metrics
- `kubectl` - K8s workloads

### Example Output

```markdown
## Resource Right-Sizing Report

### EC2 Instance Analysis (Last 30 Days)

| Instance | Current | CPU Avg | Mem Avg | Recommended | Savings |
|----------|---------|---------|---------|-------------|---------|
| api-prod-1 | m5.4xlarge | 15% | 22% | m5.xlarge | 75% |
| api-prod-2 | m5.4xlarge | 18% | 25% | m5.xlarge | 75% |
| worker-1 | c5.2xlarge | 45% | 30% | c5.xlarge | 50% |
| db-replica | r5.4xlarge | 12% | 65% | r5.2xlarge | 50% |

### Detailed Analysis: api-prod-1

**Current Configuration**:
- Type: m5.4xlarge (16 vCPU, 64 GB RAM)
- Monthly Cost: $560

**Utilization Metrics**:
```
CPU Usage (30 days):
  Average: 15%
  P95: 32%
  Peak: 45%

Memory Usage (30 days):
  Average: 22%
  P95: 38%
  Peak: 48%
```

**Recommendation**: m5.xlarge (4 vCPU, 16 GB RAM)
- Headroom: 2x peak CPU, 2x peak memory
- Monthly Cost: $140
- Savings: $420/month (75%)

### Kubernetes Workload Analysis

| Deployment | Requested | Used (Avg) | Used (P95) | Recommendation |
|------------|-----------|------------|------------|----------------|
| api-server | 2000m CPU | 380m | 650m | 800m CPU |
| worker | 4Gi Mem | 1.2Gi | 2.1Gi | 2.5Gi Mem |
| scheduler | 1000m CPU | 50m | 120m | 200m CPU |

### Resource Optimization Summary

| Category | Current | Recommended | Monthly Savings |
|----------|---------|-------------|-----------------|
| EC2 Instances | $4,200 | $1,800 | $2,400 |
| RDS | $3,100 | $1,900 | $1,200 |
| K8s Requests | N/A | -60% | Better bin packing |
| **Total** | **$7,300** | **$3,700** | **$3,600** |

### Implementation Steps

1. **Test in staging** (1 week)
   - Resize staging instances
   - Run load tests
   - Verify performance

2. **Gradual rollout** (2 weeks)
   - Resize 1 instance at a time
   - Monitor for 24 hours
   - Proceed if stable

3. **Full implementation** (1 week)
   - Complete remaining instances
   - Update auto-scaling configs
   - Document new baselines
```

---

## cloud-migrator

Plans and assists with cloud migrations.

### Usage

```bash
# Assess migration readiness
aofctl run agent library://cloud/cloud-migrator \
  --prompt "Assess our on-prem workloads for AWS migration"

# Plan migration
aofctl run agent library://cloud/cloud-migrator \
  --prompt "Create migration plan for the legacy database"
```

### Capabilities

- Workload assessment
- Dependency mapping
- Cost estimation
- Migration strategy selection
- Risk identification
- Timeline planning

### Tools Used
- Multiple cloud and infrastructure tools

### Example Output

```markdown
## Migration Assessment Report

### Source Environment

| Component | Type | Size | Dependencies |
|-----------|------|------|--------------|
| Web App | VM (8 vCPU, 32GB) | 500 GB | DB, Cache, API |
| API Server | VM (4 vCPU, 16GB) | 200 GB | DB, Queue |
| Database | PostgreSQL 13 | 2 TB | None |
| Cache | Redis Cluster | 64 GB | None |
| Queue | RabbitMQ | 50 GB | None |

### Migration Strategies

| Component | Strategy | Target | Risk | Effort |
|-----------|----------|--------|------|--------|
| Web App | Rehost | EC2 | Low | 1 week |
| API Server | Rehost | EC2 | Low | 1 week |
| Database | Replatform | RDS | Medium | 2 weeks |
| Cache | Replatform | ElastiCache | Low | 3 days |
| Queue | Replace | SQS/SNS | Medium | 2 weeks |

### Cost Comparison

| Component | On-Prem/Month | AWS/Month | Change |
|-----------|---------------|-----------|--------|
| Compute | $2,400 | $1,800 | -25% |
| Database | $1,200 | $1,500 | +25% |
| Cache | $400 | $350 | -12% |
| Queue | $200 | $50 | -75% |
| Network | $600 | $400 | -33% |
| **Total** | **$4,800** | **$4,100** | **-15%** |

### Migration Timeline

```
Week 1-2: Setup & Cache Migration
  - Set up AWS landing zone
  - Migrate Redis to ElastiCache
  - Verify cache connectivity

Week 3-4: Database Migration
  - Set up RDS PostgreSQL
  - Configure DMS replication
  - Test failover procedure

Week 5: Application Migration
  - Deploy Web App to EC2
  - Deploy API Server to EC2
  - Configure load balancers

Week 6: Queue Migration & Cutover
  - Implement SQS/SNS
  - Update application configs
  - Execute production cutover

Week 7: Validation & Optimization
  - Performance testing
  - Cost optimization
  - Documentation
```

### Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Data loss during DB migration | Low | High | DMS with validation |
| Downtime during cutover | Medium | Medium | Blue-green deployment |
| Performance degradation | Low | Medium | Right-sizing, load tests |
| Cost overrun | Medium | Low | Reserved instances |
```

---

## drift-detector

Detects infrastructure drift between declared and actual state.

### Usage

```bash
# Detect Terraform drift
aofctl run agent library://cloud/drift-detector \
  --prompt "Check for drift in our Terraform-managed infrastructure"

# Kubernetes drift
aofctl run agent library://cloud/drift-detector \
  --prompt "Find differences between Git and actual K8s state"
```

### Capabilities

- Terraform state comparison
- Kubernetes manifest drift
- CloudFormation drift detection
- Configuration comparison
- Remediation recommendations
- Drift history tracking

### Tools Used
- `terraform` - Terraform operations
- `aws_cloudformation` - CloudFormation
- `kubectl` - Kubernetes state

### Example Output

```markdown
## Infrastructure Drift Report

### Summary

| Provider | Resources | Drifted | % Drift |
|----------|-----------|---------|---------|
| Terraform | 145 | 8 | 5.5% |
| Kubernetes | 234 | 12 | 5.1% |
| CloudFormation | 45 | 2 | 4.4% |

### Terraform Drift Details

#### 1. aws_security_group.api_server
**Declared**:
```hcl
ingress {
  from_port   = 443
  to_port     = 443
  protocol    = "tcp"
  cidr_blocks = ["10.0.0.0/8"]
}
```

**Actual**:
```hcl
ingress {
  from_port   = 443
  to_port     = 443
  protocol    = "tcp"
  cidr_blocks = ["10.0.0.0/8", "0.0.0.0/0"]  # ← ADDED
}
```

**Risk**: HIGH - Security group allows public access
**Cause**: Manual AWS console change
**Remediation**: `terraform apply` or remove manual rule

#### 2. aws_instance.worker
**Declared**: instance_type = "m5.large"
**Actual**: instance_type = "m5.xlarge"

**Risk**: LOW - Instance larger than declared
**Cause**: Manual resize for debugging
**Remediation**: Update Terraform or resize instance

### Kubernetes Drift Details

| Resource | Field | Git State | Cluster State | Risk |
|----------|-------|-----------|---------------|------|
| deploy/api | replicas | 3 | 5 | LOW |
| deploy/api | image | v2.0.3 | v2.0.5 | MEDIUM |
| cm/config | LOG_LEVEL | info | debug | LOW |
| secret/db | (changed) | - | - | HIGH |

### Drift Remediation

**Option 1: Reconcile to Declared State**
```bash
# Terraform
terraform apply -auto-approve

# Kubernetes (GitOps)
argocd app sync production --prune
```

**Option 2: Update Declared State**
```bash
# Import actual state to Terraform
terraform import aws_security_group.api_server sg-xxx

# Update Git with actual K8s state
kubectl get deploy api -o yaml > deploy-api.yaml
git add deploy-api.yaml && git commit -m "Sync actual state"
```

### Prevention Recommendations

1. **Enable drift detection alerts**
   - Terraform Cloud/Enterprise drift detection
   - ArgoCD auto-sync with notifications

2. **Restrict manual changes**
   - AWS SCPs preventing console changes
   - K8s admission webhooks for GitOps

3. **Regular drift audits**
   - Weekly automated drift scans
   - Monthly manual review of changes
```

---

## Environment Setup

```bash
# AWS
export AWS_REGION=us-east-1
export AWS_PROFILE=production

# GCP
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/credentials.json
export GCP_PROJECT=my-project

# Terraform
export TF_VAR_environment=production

# Kubernetes
export KUBECONFIG=~/.kube/config
```

## Next Steps

- [Kubernetes Agents](./kubernetes.md) - K8s operations
- [Security Agents](./security.md) - Security scanning
- [Cost Optimization Guide](../guides/cost-optimization.md)
