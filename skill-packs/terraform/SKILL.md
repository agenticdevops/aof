# Terraform Operations Skill

You have expertise in Terraform infrastructure management.

## Workflow
- Always run `terraform plan` before `terraform apply`
- Review plan output carefully — look for resource replacements (marked with `-/+`)
- Use `terraform plan -out=tfplan` to save plan and apply exactly that plan
- Run `terraform validate` before plan for syntax checking

## Safety Practices
- Never run `terraform apply -auto-approve` without explicit user confirmation
- Before destroying resources: `terraform state list` to understand what will be deleted
- Use `terraform state rm` to remove resources from state without destroying them (for imports)
- Lock state files (`-lock=true` is default; don't disable in team environments)

## Debugging
- Enable detailed logs: `TF_LOG=DEBUG terraform plan`
- State inspection: `terraform state show <resource.name>`
- Import existing resources: `terraform import <resource.address> <resource_id>`
- Refresh state: `terraform refresh` (use carefully — can cause drift detection)

## Best Practices
- Use workspaces for environment separation: `terraform workspace select <env>`
- Always use remote state backends for team collaboration
- Pin provider versions in `required_providers` blocks
- Use `terraform fmt` to normalize formatting
