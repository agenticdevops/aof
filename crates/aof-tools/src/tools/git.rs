//! Git Tools
//!
//! Tools for Git repository operations.
//!
//! ## Available Tools
//!
//! - `git_status` - Show working tree status
//! - `git_diff` - Show changes
//! - `git_log` - Show commit history
//! - `git_commit` - Create commits
//! - `git_branch` - List/create branches
//! - `git_checkout` - Switch branches
//! - `git_pull` - Pull changes
//! - `git_push` - Push changes
//!
//! ## Prerequisites
//!
//! - Git must be installed and in PATH
//! - Repository must be initialized
//!
//! ## MCP Alternative
//!
//! For MCP-based Git operations, use the Git MCP server.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{execute_command, create_schema, tool_config_with_timeout};

/// Collection of all Git tools
pub struct GitTools;

impl GitTools {
    /// Get all Git tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(GitStatusTool::new()),
            Box::new(GitDiffTool::new()),
            Box::new(GitLogTool::new()),
            Box::new(GitCommitTool::new()),
            Box::new(GitBranchTool::new()),
            Box::new(GitCheckoutTool::new()),
            Box::new(GitPullTool::new()),
            Box::new(GitPushTool::new()),
        ]
    }

    /// Check if git is available
    pub fn is_available() -> bool {
        which::which("git").is_ok()
    }
}

// ============================================================================
// Git Status Tool
// ============================================================================

/// Show working tree status
pub struct GitStatusTool {
    config: ToolConfig,
}

impl GitStatusTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Repository path (default: current directory)",
                    "default": "."
                },
                "short": {
                    "type": "boolean",
                    "description": "Use short format",
                    "default": false
                },
                "porcelain": {
                    "type": "boolean",
                    "description": "Machine-readable output",
                    "default": true
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "git_status",
                "Show the working tree status. Lists modified, staged, and untracked files.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for GitStatusTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitStatusTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path").unwrap_or_else(|_| ".".to_string());
        let short: bool = input.get_arg("short").unwrap_or(false);
        let porcelain: bool = input.get_arg("porcelain").unwrap_or(true);

        let mut args = vec!["status"];

        if short {
            args.push("-s");
        }

        if porcelain {
            args.push("--porcelain=v1");
        }

        debug!(args = ?args, path = %path, "Executing git status");

        let result = execute_command("git", &args, Some(&path), 30).await;

        match result {
            Ok(output) => {
                if output.success {
                    // Parse porcelain output
                    let files: Vec<serde_json::Value> = if porcelain {
                        output
                            .stdout
                            .lines()
                            .filter(|l| !l.is_empty())
                            .map(|line| {
                                let status = &line[..2];
                                let file = &line[3..];
                                serde_json::json!({
                                    "status": status.trim(),
                                    "file": file
                                })
                            })
                            .collect()
                    } else {
                        vec![]
                    };

                    Ok(ToolResult::success(serde_json::json!({
                        "files": files,
                        "count": files.len(),
                        "clean": files.is_empty(),
                        "raw": output.stdout
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "git status failed: {}",
                        output.stderr
                    )))
                }
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Git Diff Tool
// ============================================================================

/// Show changes
pub struct GitDiffTool {
    config: ToolConfig,
}

impl GitDiffTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Repository path",
                    "default": "."
                },
                "file": {
                    "type": "string",
                    "description": "Specific file to diff"
                },
                "staged": {
                    "type": "boolean",
                    "description": "Show staged changes",
                    "default": false
                },
                "commit": {
                    "type": "string",
                    "description": "Compare with specific commit"
                },
                "stat": {
                    "type": "boolean",
                    "description": "Show diffstat",
                    "default": false
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "git_diff",
                "Show changes between commits, commit and working tree, etc.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitDiffTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitDiffTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path").unwrap_or_else(|_| ".".to_string());
        let file: Option<String> = input.get_arg("file").ok();
        let staged: bool = input.get_arg("staged").unwrap_or(false);
        let commit: Option<String> = input.get_arg("commit").ok();
        let stat: bool = input.get_arg("stat").unwrap_or(false);

        let mut args = vec!["diff".to_string()];

        if staged {
            args.push("--staged".to_string());
        }

        if let Some(ref c) = commit {
            args.push(c.clone());
        }

        if stat {
            args.push("--stat".to_string());
        }

        if let Some(ref f) = file {
            args.push("--".to_string());
            args.push(f.clone());
        }

        debug!(args = ?args, path = %path, "Executing git diff");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("git", &args_str, Some(&path), 60).await;

        match result {
            Ok(output) => {
                if output.success {
                    Ok(ToolResult::success(serde_json::json!({
                        "diff": output.stdout,
                        "has_changes": !output.stdout.is_empty(),
                        "lines": output.stdout.lines().count()
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "git diff failed: {}",
                        output.stderr
                    )))
                }
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Git Log Tool
// ============================================================================

/// Show commit history
pub struct GitLogTool {
    config: ToolConfig,
}

impl GitLogTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Repository path",
                    "default": "."
                },
                "count": {
                    "type": "integer",
                    "description": "Number of commits to show",
                    "default": 10
                },
                "oneline": {
                    "type": "boolean",
                    "description": "One line per commit",
                    "default": true
                },
                "author": {
                    "type": "string",
                    "description": "Filter by author"
                },
                "since": {
                    "type": "string",
                    "description": "Show commits since date (e.g., '2024-01-01')"
                },
                "file": {
                    "type": "string",
                    "description": "Show commits for specific file"
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "git_log",
                "Show commit logs.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for GitLogTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitLogTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path").unwrap_or_else(|_| ".".to_string());
        let count: i32 = input.get_arg("count").unwrap_or(10);
        let oneline: bool = input.get_arg("oneline").unwrap_or(true);
        let author: Option<String> = input.get_arg("author").ok();
        let since: Option<String> = input.get_arg("since").ok();
        let file: Option<String> = input.get_arg("file").ok();

        let mut args = vec!["log".to_string(), format!("-{}", count)];

        if oneline {
            args.push("--oneline".to_string());
        } else {
            args.push("--format=%H|%an|%ae|%at|%s".to_string());
        }

        if let Some(ref a) = author {
            args.push(format!("--author={}", a));
        }

        if let Some(ref s) = since {
            args.push(format!("--since={}", s));
        }

        if let Some(ref f) = file {
            args.push("--".to_string());
            args.push(f.clone());
        }

        debug!(args = ?args, path = %path, "Executing git log");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("git", &args_str, Some(&path), 30).await;

        match result {
            Ok(output) => {
                if output.success {
                    let commits: Vec<serde_json::Value> = if oneline {
                        output
                            .stdout
                            .lines()
                            .filter(|l| !l.is_empty())
                            .map(|line| {
                                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                                serde_json::json!({
                                    "hash": parts.first().unwrap_or(&""),
                                    "message": parts.get(1).unwrap_or(&"")
                                })
                            })
                            .collect()
                    } else {
                        output
                            .stdout
                            .lines()
                            .filter(|l| !l.is_empty())
                            .filter_map(|line| {
                                let parts: Vec<&str> = line.split('|').collect();
                                if parts.len() >= 5 {
                                    Some(serde_json::json!({
                                        "hash": parts[0],
                                        "author": parts[1],
                                        "email": parts[2],
                                        "timestamp": parts[3],
                                        "message": parts[4]
                                    }))
                                } else {
                                    None
                                }
                            })
                            .collect()
                    };

                    Ok(ToolResult::success(serde_json::json!({
                        "commits": commits,
                        "count": commits.len()
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "git log failed: {}",
                        output.stderr
                    )))
                }
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Git Commit Tool
// ============================================================================

/// Create commits
pub struct GitCommitTool {
    config: ToolConfig,
}

impl GitCommitTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "message": {
                    "type": "string",
                    "description": "Commit message"
                },
                "path": {
                    "type": "string",
                    "description": "Repository path",
                    "default": "."
                },
                "all": {
                    "type": "boolean",
                    "description": "Stage all modified files",
                    "default": false
                },
                "files": {
                    "type": "array",
                    "description": "Specific files to commit",
                    "items": { "type": "string" }
                }
            }),
            vec!["message"],
        );

        Self {
            config: tool_config_with_timeout(
                "git_commit",
                "Record changes to the repository.",
                parameters,
                60,
            ),
        }
    }
}

impl Default for GitCommitTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitCommitTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let message: String = input.get_arg("message")?;
        let path: String = input.get_arg("path").unwrap_or_else(|_| ".".to_string());
        let all: bool = input.get_arg("all").unwrap_or(false);
        let files: Vec<String> = input.get_arg("files").unwrap_or_default();

        // First, add files if specified
        if !files.is_empty() {
            let mut add_args = vec!["add".to_string()];
            add_args.extend(files.clone());
            let add_args_str: Vec<&str> = add_args.iter().map(|s| s.as_str()).collect();
            let _ = execute_command("git", &add_args_str, Some(&path), 30).await;
        }

        let mut args = vec!["commit".to_string()];

        if all {
            args.push("-a".to_string());
        }

        args.push("-m".to_string());
        args.push(message.clone());

        debug!(args = ?args, path = %path, "Executing git commit");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("git", &args_str, Some(&path), 60).await;

        match result {
            Ok(output) => {
                if output.success {
                    // Get the commit hash
                    let hash_result = execute_command(
                        "git",
                        &["rev-parse", "HEAD"],
                        Some(&path),
                        10,
                    )
                    .await;
                    let hash = hash_result
                        .map(|o| o.stdout.trim().to_string())
                        .unwrap_or_default();

                    Ok(ToolResult::success(serde_json::json!({
                        "committed": true,
                        "hash": hash,
                        "message": message,
                        "output": output.stdout
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "git commit failed: {}",
                        output.stderr
                    )))
                }
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Git Branch Tool
// ============================================================================

/// List or create branches
pub struct GitBranchTool {
    config: ToolConfig,
}

impl GitBranchTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Repository path",
                    "default": "."
                },
                "name": {
                    "type": "string",
                    "description": "Branch name to create"
                },
                "delete": {
                    "type": "string",
                    "description": "Branch name to delete"
                },
                "all": {
                    "type": "boolean",
                    "description": "List all branches including remotes",
                    "default": false
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "git_branch",
                "List, create, or delete branches.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for GitBranchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitBranchTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path").unwrap_or_else(|_| ".".to_string());
        let name: Option<String> = input.get_arg("name").ok();
        let delete: Option<String> = input.get_arg("delete").ok();
        let all: bool = input.get_arg("all").unwrap_or(false);

        let args: Vec<&str> = if let Some(ref n) = name {
            vec!["branch", n]
        } else if let Some(ref d) = delete {
            vec!["branch", "-d", d]
        } else if all {
            vec!["branch", "-a"]
        } else {
            vec!["branch"]
        };

        debug!(args = ?args, path = %path, "Executing git branch");

        let result = execute_command("git", &args, Some(&path), 30).await;

        match result {
            Ok(output) => {
                if output.success {
                    if name.is_some() {
                        Ok(ToolResult::success(serde_json::json!({
                            "created": true,
                            "branch": name
                        })))
                    } else if delete.is_some() {
                        Ok(ToolResult::success(serde_json::json!({
                            "deleted": true,
                            "branch": delete
                        })))
                    } else {
                        let branches: Vec<serde_json::Value> = output
                            .stdout
                            .lines()
                            .filter(|l| !l.is_empty())
                            .map(|line| {
                                let current = line.starts_with('*');
                                let name = line.trim_start_matches('*').trim();
                                serde_json::json!({
                                    "name": name,
                                    "current": current
                                })
                            })
                            .collect();

                        Ok(ToolResult::success(serde_json::json!({
                            "branches": branches,
                            "count": branches.len()
                        })))
                    }
                } else {
                    Ok(ToolResult::error(format!(
                        "git branch failed: {}",
                        output.stderr
                    )))
                }
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Git Checkout Tool
// ============================================================================

/// Switch branches
pub struct GitCheckoutTool {
    config: ToolConfig,
}

impl GitCheckoutTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "branch": {
                    "type": "string",
                    "description": "Branch to checkout"
                },
                "path": {
                    "type": "string",
                    "description": "Repository path",
                    "default": "."
                },
                "create": {
                    "type": "boolean",
                    "description": "Create branch if it doesn't exist",
                    "default": false
                },
                "file": {
                    "type": "string",
                    "description": "Checkout specific file from HEAD"
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "git_checkout",
                "Switch branches or restore working tree files.",
                parameters,
                30,
            ),
        }
    }
}

impl Default for GitCheckoutTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitCheckoutTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let branch: Option<String> = input.get_arg("branch").ok();
        let path: String = input.get_arg("path").unwrap_or_else(|_| ".".to_string());
        let create: bool = input.get_arg("create").unwrap_or(false);
        let file: Option<String> = input.get_arg("file").ok();

        if branch.is_none() && file.is_none() {
            return Ok(ToolResult::error("Either 'branch' or 'file' is required"));
        }

        let args: Vec<String> = if let Some(ref f) = file {
            vec!["checkout".to_string(), "--".to_string(), f.clone()]
        } else if let Some(ref b) = branch {
            if create {
                vec!["checkout".to_string(), "-b".to_string(), b.clone()]
            } else {
                vec!["checkout".to_string(), b.clone()]
            }
        } else {
            return Ok(ToolResult::error("Invalid arguments"));
        };

        debug!(args = ?args, path = %path, "Executing git checkout");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("git", &args_str, Some(&path), 30).await;

        match result {
            Ok(output) => {
                if output.success {
                    Ok(ToolResult::success(serde_json::json!({
                        "success": true,
                        "branch": branch,
                        "file": file,
                        "created": create
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "git checkout failed: {}",
                        output.stderr
                    )))
                }
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Git Pull Tool
// ============================================================================

/// Pull changes
pub struct GitPullTool {
    config: ToolConfig,
}

impl GitPullTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Repository path",
                    "default": "."
                },
                "remote": {
                    "type": "string",
                    "description": "Remote name",
                    "default": "origin"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch to pull"
                },
                "rebase": {
                    "type": "boolean",
                    "description": "Rebase instead of merge",
                    "default": false
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "git_pull",
                "Fetch from and integrate with another repository or branch.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for GitPullTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitPullTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path").unwrap_or_else(|_| ".".to_string());
        let remote: String = input.get_arg("remote").unwrap_or_else(|_| "origin".to_string());
        let branch: Option<String> = input.get_arg("branch").ok();
        let rebase: bool = input.get_arg("rebase").unwrap_or(false);

        let mut args = vec!["pull".to_string()];

        if rebase {
            args.push("--rebase".to_string());
        }

        args.push(remote.clone());

        if let Some(ref b) = branch {
            args.push(b.clone());
        }

        debug!(args = ?args, path = %path, "Executing git pull");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("git", &args_str, Some(&path), 120).await;

        match result {
            Ok(output) => {
                if output.success {
                    Ok(ToolResult::success(serde_json::json!({
                        "pulled": true,
                        "remote": remote,
                        "branch": branch,
                        "output": output.stdout
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "git pull failed: {}",
                        output.stderr
                    )))
                }
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Git Push Tool
// ============================================================================

/// Push changes
pub struct GitPushTool {
    config: ToolConfig,
}

impl GitPushTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Repository path",
                    "default": "."
                },
                "remote": {
                    "type": "string",
                    "description": "Remote name",
                    "default": "origin"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch to push"
                },
                "set_upstream": {
                    "type": "boolean",
                    "description": "Set upstream tracking",
                    "default": false
                },
                "tags": {
                    "type": "boolean",
                    "description": "Push tags",
                    "default": false
                }
            }),
            vec![],
        );

        Self {
            config: tool_config_with_timeout(
                "git_push",
                "Update remote refs along with associated objects.",
                parameters,
                120,
            ),
        }
    }
}

impl Default for GitPushTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitPushTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path").unwrap_or_else(|_| ".".to_string());
        let remote: String = input.get_arg("remote").unwrap_or_else(|_| "origin".to_string());
        let branch: Option<String> = input.get_arg("branch").ok();
        let set_upstream: bool = input.get_arg("set_upstream").unwrap_or(false);
        let tags: bool = input.get_arg("tags").unwrap_or(false);

        let mut args = vec!["push".to_string()];

        if set_upstream {
            args.push("-u".to_string());
        }

        if tags {
            args.push("--tags".to_string());
        }

        args.push(remote.clone());

        if let Some(ref b) = branch {
            args.push(b.clone());
        }

        debug!(args = ?args, path = %path, "Executing git push");

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = execute_command("git", &args_str, Some(&path), 120).await;

        match result {
            Ok(output) => {
                if output.success {
                    Ok(ToolResult::success(serde_json::json!({
                        "pushed": true,
                        "remote": remote,
                        "branch": branch,
                        "output": format!("{}{}", output.stdout, output.stderr)
                    })))
                } else {
                    Ok(ToolResult::error(format!(
                        "git push failed: {}",
                        output.stderr
                    )))
                }
            }
            Err(e) => Ok(ToolResult::error(e)),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}
