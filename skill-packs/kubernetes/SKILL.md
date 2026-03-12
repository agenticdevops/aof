# Kubernetes Operations Skill

You have expertise in Kubernetes cluster management and workload operations.

## CLI Usage
- Use `kubectl` with `-o json` or `-o yaml` for detailed output
- Always specify `-n <namespace>` or use `--all-namespaces` (-A)
- Use `--context` to target specific clusters when multiple exist
- Prefer `kubectl get` before `kubectl delete` to confirm what will be affected

## Diagnostic Patterns
- Pod issues: `kubectl describe pod <name> -n <ns>` — look at Events section
- Crashing: `kubectl logs <pod> -n <ns> --previous` for prior container logs
- Pending pods: Check `kubectl describe node` for resource pressure
- Services: `kubectl get endpoints <svc>` to verify Pod selection
- Resource usage: `kubectl top pods -n <ns> --sort-by=memory`

## Safety Practices
- Before scaling or deleting, list current state: `kubectl get <resource> -n <ns>`
- Use `--dry-run=client -o yaml` to preview changes before applying
- Confirm namespace before delete operations — never use `default` namespace for production
- For rollbacks: `kubectl rollout undo deployment/<name> -n <ns>`

## Common Commands
- Restart: `kubectl rollout restart deployment/<name> -n <ns>`
- Force delete stuck pods: `kubectl delete pod <name> --grace-period=0 --force -n <ns>`
- Port-forward: `kubectl port-forward svc/<name> <local>:<remote> -n <ns>`
- Exec into pod: `kubectl exec -it <pod> -n <ns> -- /bin/sh`
