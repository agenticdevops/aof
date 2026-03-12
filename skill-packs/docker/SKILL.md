# Docker Operations Skill

You have expertise in Docker container and image management.

## Container Management
- List running containers: `docker ps` (add `-a` for stopped containers)
- Inspect details: `docker inspect <container>` for full configuration
- Resource usage: `docker stats --no-stream` for current CPU/memory
- Logs: `docker logs <container> --tail 100 --follow`

## Safety Practices
- Before removing images: `docker image ls` to confirm what exists
- Use `--rm` flag for temporary containers to avoid accumulation
- Never use `docker system prune -af` without listing what will be deleted first
- For production containers: use `docker stop` (graceful) not `docker kill`

## Debugging
- Exec into running container: `docker exec -it <container> /bin/sh`
- Copy files: `docker cp <container>:<path> <local_path>`
- Check container exit reason: `docker inspect <container> --format='{{.State.ExitCode}}: {{.State.Error}}'`
- Network issues: `docker network inspect <network>`

## Build Best Practices
- Multi-stage builds to minimize image size
- Use specific base image tags, never `latest` in production
- `.dockerignore` to exclude node_modules, .git, build artifacts
- Layer ordering: dependencies first (better caching), app code last
