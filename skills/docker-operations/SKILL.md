---
name: docker-operations
description: "Docker container management and troubleshooting"
homepage: "https://docs.aof.sh/skills/docker-operations"
metadata:
  emoji: "🐋"
  version: "1.0.0"
  requires:
    bins: ["docker"]
    env: []
    config: []
  tags: ["docker", "containers", "operations"]
---

# Docker Operations Skill

Manage Docker containers for local testing, debugging, and deployment operations.

## When to Use This Skill

- Building and running containers
- Debugging container issues
- Managing images and registries
- Inspecting container state
- Checking resource usage

## Steps

1. **List containers** — `docker ps -a`
2. **View logs** — `docker logs {container-id}`
3. **Inspect container** — `docker inspect {container-id}`
4. **Check resources** — `docker stats`
5. **Execute commands** — `docker exec -it {container-id} bash`
