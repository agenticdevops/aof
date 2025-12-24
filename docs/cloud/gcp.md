---
sidebar_position: 4
---

# GCP Tools

AOF provides 8 GCP tools covering compute, storage, Kubernetes, IAM, and more.

## Prerequisites

- Google Cloud SDK installed
- Authenticated via `gcloud auth login`

```bash
# Install Google Cloud SDK
curl https://sdk.cloud.google.com | bash
exec -l $SHELL

# Initialize and authenticate
gcloud init
gcloud auth login

# Set default project
gcloud config set project your-project-id
```

## Available Tools

| Tool | Service | Description |
|------|---------|-------------|
| `gcp_compute` | Compute Engine | VM instance management |
| `gcp_storage` | Cloud Storage | Bucket and object operations |
| `gcp_gke` | GKE | Kubernetes cluster management |
| `gcp_iam` | IAM | Identity and access management |
| `gcp_logging` | Cloud Logging | Log querying and management |
| `gcp_pubsub` | Pub/Sub | Messaging operations |
| `gcp_sql` | Cloud SQL | Database management |
| `gcp_functions` | Cloud Functions | Serverless function operations |

## Tool Reference

### gcp_compute

Compute Engine instance operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `instances-list`, `instances-describe`, `instances-start`, `instances-stop`, `instances-reset`, `instances-delete`, `disks-list`, `machine-types-list` |
| `instance_name` | string | No | Instance name |
| `zone` | string | No | Zone (e.g., us-central1-a) |
| `project` | string | No | Project ID |
| `filter` | string | No | Filter expression |
| `format` | string | No | Output format: `json`, `text`, `yaml` |

**Example:**

```yaml
tools:
  - gcp_compute

# "List all instances in project my-project"
# "Stop instance web-server in zone us-central1-a"
# "List available machine types in us-central1-a"
```

### gcp_storage

Cloud Storage bucket and object operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `ls`, `cp`, `rm`, `mb`, `rb`, `mv`, `rsync` |
| `source` | string | No | Source path (local or gs://) |
| `destination` | string | No | Destination path |
| `recursive` | boolean | No | Recursive operation |
| `project` | string | No | Project ID |

**Example:**

```yaml
tools:
  - gcp_storage

# "List all buckets"
# "Copy backup.tar.gz to gs://my-bucket/backups/"
# "Sync local ./dist to gs://my-bucket/static/"
```

### gcp_gke

Google Kubernetes Engine operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `clusters-list`, `clusters-describe`, `clusters-get-credentials`, `clusters-create`, `clusters-delete`, `clusters-upgrade`, `node-pools-list`, `node-pools-describe` |
| `cluster_name` | string | No | Cluster name |
| `zone` | string | No | Zone (for zonal clusters) |
| `region` | string | No | Region (for regional clusters) |
| `project` | string | No | Project ID |
| `format` | string | No | Output format |

**Example:**

```yaml
tools:
  - gcp_gke

# "List all GKE clusters"
# "Get credentials for cluster prod-cluster in us-central1"
# "Describe node pools for cluster staging-cluster"
```

### gcp_iam

IAM and service account operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `roles-list`, `roles-describe`, `service-accounts-list`, `service-accounts-describe`, `service-accounts-keys-list`, `service-accounts-keys-create` |
| `role_name` | string | No | Role name |
| `service_account` | string | No | Service account email |
| `project` | string | No | Project ID |
| `format` | string | No | Output format |

**Example:**

```yaml
tools:
  - gcp_iam

# "List all service accounts"
# "List keys for service account deploy@my-project.iam.gserviceaccount.com"
# "Describe role roles/compute.admin"
```

### gcp_logging

Cloud Logging operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `read`, `logs-list`, `logs-delete`, `sinks-list` |
| `filter` | string | No | Log filter expression |
| `log_name` | string | No | Log name |
| `limit` | integer | No | Max entries (default: 100) |
| `project` | string | No | Project ID |
| `format` | string | No | Output format |

**Example:**

```yaml
tools:
  - gcp_logging

# "Read logs from the last hour with severity ERROR"
# "List all log names in project"
# "Read logs for resource.type=\"gke_cluster\""
```

### gcp_pubsub

Pub/Sub messaging operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `topics-list`, `topics-describe`, `topics-create`, `topics-delete`, `topics-publish`, `subscriptions-list`, `subscriptions-describe`, `subscriptions-create`, `subscriptions-delete` |
| `topic_name` | string | No | Topic name |
| `subscription_name` | string | No | Subscription name |
| `message` | string | No | Message to publish |
| `project` | string | No | Project ID |
| `format` | string | No | Output format |

**Example:**

```yaml
tools:
  - gcp_pubsub

# "List all topics"
# "Publish message to topic events"
# "List subscriptions for topic notifications"
```

### gcp_sql

Cloud SQL database operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `instances-list`, `instances-describe`, `instances-create`, `instances-delete`, `instances-restart`, `instances-patch`, `backups-list`, `backups-describe`, `backups-create` |
| `instance_name` | string | No | Instance name |
| `backup_id` | string | No | Backup ID |
| `project` | string | No | Project ID |
| `format` | string | No | Output format |

**Example:**

```yaml
tools:
  - gcp_sql

# "List all Cloud SQL instances"
# "Create backup of instance production-db"
# "Describe instance staging-mysql"
```

### gcp_functions

Cloud Functions operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `list`, `describe`, `deploy`, `delete`, `call`, `logs-read` |
| `function_name` | string | No | Function name |
| `data` | string | No | JSON data for function call |
| `region` | string | No | Region |
| `project` | string | No | Project ID |
| `format` | string | No | Output format |

**Example:**

```yaml
tools:
  - gcp_functions

# "List all Cloud Functions"
# "Call function process-order with data {\"orderId\": \"123\"}"
# "Read logs for function email-sender"
```

## Example Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: gcp-ops
spec:
  model: google:gemini-2.5-flash
  tools:
    - gcp_compute
    - gcp_gke
    - gcp_storage
    - gcp_logging

  environment:
    GOOGLE_CLOUD_PROJECT: "${GOOGLE_CLOUD_PROJECT}"
    CLOUDSDK_COMPUTE_ZONE: "us-central1-a"

  system_prompt: |
    You are a GCP cloud operations specialist.
    Help manage Compute Engine, GKE, Storage, and Logging.

    ## Guidelines
    - Always specify project and zone/region when required
    - Use JSON format for data analysis
    - Confirm destructive operations before proceeding
```

## Common Patterns

### Working with Projects

```yaml
environment:
  GOOGLE_CLOUD_PROJECT: "my-project"

# Or specify per operation
"List instances in project dev-project"
```

### Zone vs Region

```yaml
# Zonal resources (VMs, zonal GKE)
"List instances in zone us-central1-a"

# Regional resources (regional GKE, Cloud SQL)
"Describe cluster prod-cluster in region us-central1"
```

### Log Filtering

```yaml
# Filter by severity
"Read logs with severity >= ERROR"

# Filter by resource
"Read logs for resource.type=\"gke_cluster\" AND resource.labels.cluster_name=\"prod\""

# Filter by time
"Read logs from the last 2 hours"
```

### Service Account Management

```yaml
# List and audit service accounts
"List all service accounts"
"List keys for service account"
# Identify keys older than 90 days for rotation
```
