---
sidebar_position: 3
---

# Azure Tools

AOF provides 8 Azure tools covering compute, storage, containers, networking, and security.

## Prerequisites

- Azure CLI installed
- Authenticated via `az login`

```bash
# Install Azure CLI
curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash

# Login
az login

# Set subscription
az account set --subscription "your-subscription-id"
```

## Available Tools

| Tool | Service | Description |
|------|---------|-------------|
| `azure_vm` | Virtual Machines | VM management |
| `azure_storage` | Storage | Blob and account operations |
| `azure_aks` | AKS | Kubernetes service management |
| `azure_network` | Networking | VNet, NSG, and IP management |
| `azure_resource` | Resource Manager | Resource group operations |
| `azure_keyvault` | Key Vault | Secret management |
| `azure_monitor` | Monitor | Metrics and alerts |
| `azure_acr` | Container Registry | Container image management |

## Tool Reference

### azure_vm

Virtual Machine operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `list`, `show`, `start`, `stop`, `restart`, `delete` |
| `name` | string | No | VM name |
| `resource_group` | string | No | Resource group name |
| `subscription` | string | No | Subscription ID |
| `output` | string | No | Output format: `json`, `table`, `tsv` |

**Example:**

```yaml
tools:
  - azure_vm

# "List all VMs in resource group production-rg"
# "Stop VM web-server-01"
# "Restart all VMs in staging-rg"
```

### azure_storage

Storage account and blob operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `account-list`, `blob-list`, `blob-upload`, `blob-download` |
| `account_name` | string | No | Storage account name |
| `container_name` | string | No | Container name |
| `blob_name` | string | No | Blob name |
| `file_path` | string | No | Local file path |
| `resource_group` | string | No | Resource group name |
| `subscription` | string | No | Subscription ID |
| `output` | string | No | Output format |

**Example:**

```yaml
tools:
  - azure_storage

# "List all storage accounts"
# "List blobs in container 'backups' in account 'mystorageacct'"
# "Upload backup.tar.gz to container 'backups'"
```

### azure_aks

Azure Kubernetes Service operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `list`, `show`, `get-credentials`, `scale` |
| `name` | string | No | Cluster name |
| `resource_group` | string | No | Resource group name |
| `node_count` | integer | No | Node count for scaling |
| `nodepool_name` | string | No | Node pool name |
| `subscription` | string | No | Subscription ID |
| `output` | string | No | Output format |

**Example:**

```yaml
tools:
  - azure_aks

# "List all AKS clusters"
# "Get credentials for cluster prod-aks in prod-rg"
# "Scale nodepool 'agentpool' to 5 nodes"
```

### azure_network

Networking operations (VNet, NSG, Public IP).

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `vnet-list`, `nsg-list`, `public-ip-list` |
| `resource_group` | string | No | Resource group name |
| `subscription` | string | No | Subscription ID |
| `output` | string | No | Output format |

**Example:**

```yaml
tools:
  - azure_network

# "List all VNets in subscription"
# "List NSGs in resource group network-rg"
# "List all public IPs"
```

### azure_resource

Resource Manager operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `group-list`, `group-show`, `resource-list` |
| `resource_group` | string | No | Resource group name |
| `resource_type` | string | No | Filter by resource type |
| `subscription` | string | No | Subscription ID |
| `output` | string | No | Output format |

**Example:**

```yaml
tools:
  - azure_resource

# "List all resource groups"
# "Show details of resource group production-rg"
# "List all resources in staging-rg"
```

### azure_keyvault

Key Vault secret operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `secret-list`, `secret-show`, `secret-set` |
| `vault_name` | string | No | Key Vault name |
| `secret_name` | string | No | Secret name |
| `secret_value` | string | No | Secret value (for set) |
| `subscription` | string | No | Subscription ID |
| `output` | string | No | Output format |

**Example:**

```yaml
tools:
  - azure_keyvault

# "List secrets in vault 'prod-keyvault'"
# "Get secret 'db-password' from vault 'prod-keyvault'"
```

### azure_monitor

Azure Monitor operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `metrics-list`, `activity-log-list`, `alert-list` |
| `resource_id` | string | No | Resource ID for metrics |
| `resource_group` | string | No | Resource group name |
| `start_time` | string | No | Start time (ISO 8601) |
| `end_time` | string | No | End time (ISO 8601) |
| `subscription` | string | No | Subscription ID |
| `output` | string | No | Output format |

**Example:**

```yaml
tools:
  - azure_monitor

# "List metrics for VM /subscriptions/.../virtualMachines/my-vm"
# "Get activity log for the last 24 hours"
# "List all alerts in resource group monitoring-rg"
```

### azure_acr

Azure Container Registry operations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | Yes | `list`, `show`, `repository-list` |
| `name` | string | No | Registry name |
| `resource_group` | string | No | Resource group name |
| `subscription` | string | No | Subscription ID |
| `output` | string | No | Output format |

**Example:**

```yaml
tools:
  - azure_acr

# "List all container registries"
# "List repositories in registry 'myacr'"
```

## Example Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: azure-ops
spec:
  model: google:gemini-2.5-flash
  tools:
    - azure_vm
    - azure_aks
    - azure_storage
    - azure_keyvault

  environment:
    AZURE_SUBSCRIPTION_ID: "${AZURE_SUBSCRIPTION_ID}"

  system_prompt: |
    You are an Azure cloud operations specialist.
    Help manage VMs, AKS clusters, storage, and secrets.

    ## Guidelines
    - Always specify resource group when required
    - Confirm destructive operations before proceeding
    - Use proper output format for data analysis
```

## Common Patterns

### Working with Resource Groups

```yaml
# Most operations require resource group
"List VMs in resource group production-rg"
"Show AKS cluster in rg my-aks-rg"
```

### Managing Subscriptions

```yaml
environment:
  AZURE_SUBSCRIPTION_ID: "xxx-xxx-xxx"

# Or specify per operation
"List VMs in subscription 'dev-subscription'"
```

### Secret Management

```yaml
# Never output secret values in logs
"Check if secret 'api-key' exists in vault 'prod-vault'"
# Instead of: "Get and display secret value"
```
