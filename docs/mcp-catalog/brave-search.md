---
sidebar_position: 9
sidebar_label: Brave Search
---

# Brave Search MCP Server

Web search using the Brave Search API.

## Overview

| Property | Value |
|----------|-------|
| Package | `@modelcontextprotocol/server-brave-search` |
| Source | [GitHub](https://github.com/modelcontextprotocol/servers/tree/main/src/brave-search) |
| Transport | stdio |

## Installation

```bash
npx -y @modelcontextprotocol/server-brave-search
```

## Configuration

```yaml
mcp_servers:
  - name: brave-search
    command: npx
    args: ["-y", "@modelcontextprotocol/server-brave-search"]
    env:
      BRAVE_API_KEY: ${BRAVE_API_KEY}
```

### Getting an API Key

1. Go to https://brave.com/search/api/
2. Sign up for an account
3. Choose a plan (Free tier available)
4. Get your API key

## Tools

### brave_web_search

Search the web.

**Parameters**:
- `query` (string, required): Search query
- `count` (number, optional): Number of results (default: 10, max: 20)

**Example**:
```json
{
  "tool": "brave_web_search",
  "arguments": {
    "query": "kubernetes best practices 2024",
    "count": 5
  }
}
```

**Returns**:
```json
{
  "results": [
    {
      "title": "Kubernetes Best Practices Guide",
      "url": "https://example.com/k8s-guide",
      "description": "A comprehensive guide to K8s best practices..."
    }
  ]
}
```

### brave_local_search

Search for local businesses and places.

**Parameters**:
- `query` (string, required): Search query
- `count` (number, optional): Number of results

**Example**:
```json
{
  "tool": "brave_local_search",
  "arguments": {
    "query": "coffee shops near San Francisco"
  }
}
```

## Use Cases

### Research Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: research-agent
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: brave-search
      command: npx
      args: ["-y", "@modelcontextprotocol/server-brave-search"]
      env:
        BRAVE_API_KEY: ${BRAVE_API_KEY}
  system_prompt: |
    You research topics using web search:
    1. Search for relevant information
    2. Analyze multiple sources
    3. Synthesize findings
    4. Cite sources properly

    Always verify information from multiple sources.
```

### Documentation Finder

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: doc-finder
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: brave-search
      command: npx
      args: ["-y", "@modelcontextprotocol/server-brave-search"]
      env:
        BRAVE_API_KEY: ${BRAVE_API_KEY}
  system_prompt: |
    You find documentation and guides:
    - Search for official documentation
    - Find tutorials and examples
    - Locate API references
    - Identify community resources

    Prioritize official sources over third-party.
```

### Security Researcher

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: security-researcher
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: brave-search
      command: npx
      args: ["-y", "@modelcontextprotocol/server-brave-search"]
      env:
        BRAVE_API_KEY: ${BRAVE_API_KEY}
  system_prompt: |
    You research security vulnerabilities:
    - Search for CVE information
    - Find security advisories
    - Locate patches and fixes
    - Identify affected versions

    Focus on authoritative sources:
    - NVD (nvd.nist.gov)
    - Vendor advisories
    - MITRE CVE database
```

### Trend Analyzer

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: trend-analyzer
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: brave-search
      command: npx
      args: ["-y", "@modelcontextprotocol/server-brave-search"]
      env:
        BRAVE_API_KEY: ${BRAVE_API_KEY}
  system_prompt: |
    You analyze technology trends:
    - Search for recent developments
    - Compare competing solutions
    - Track adoption patterns
    - Identify emerging technologies

    Provide balanced analysis with pros/cons.
```

## Search Tips

### Effective Queries

```
# Specific technology
"kubernetes 1.29" new features

# Site-specific
site:github.com kubernetes operators

# File type
filetype:pdf kubernetes security

# Recent results
kubernetes best practices 2024

# Exclude terms
kubernetes -docker
```

### Query Optimization

1. **Be specific**: Include version numbers, dates
2. **Use quotes**: For exact phrases
3. **Combine terms**: Use multiple relevant keywords
4. **Filter sites**: Use `site:` for authoritative sources

## Rate Limits

Brave Search API limits depend on your plan:

| Plan | Requests/Month | Rate |
|------|----------------|------|
| Free | 2,000 | 1/second |
| Basic | 20,000 | 20/second |
| Pro | Unlimited | 50/second |

## Privacy

Brave Search is privacy-focused:
- No user tracking
- No personalized results
- Independent search index
- No search history stored

## Troubleshooting

### Invalid API Key

Verify your key:
```bash
curl -H "X-Subscription-Token: YOUR_API_KEY" \
  "https://api.search.brave.com/res/v1/web/search?q=test"
```

### Rate Limited

Response includes retry-after header:
- Wait for indicated time
- Consider upgrading plan
- Implement request queuing

### No Results

Try different query approaches:
- Remove specific terms
- Use broader keywords
- Check for typos

### Empty Response

Some queries may return no results:
- Very niche topics
- Recent events not indexed
- Restricted content
