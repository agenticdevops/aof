//! Trivy Security Scanner Tools
//!
//! Tools for container image scanning, filesystem vulnerability detection,
//! IaC configuration scanning, and SBOM generation using Aqua Security's Trivy.
//!
//! ## Available Tools
//!
//! - `trivy_image_scan` - Scan container images for vulnerabilities
//! - `trivy_fs_scan` - Scan filesystem/directory for vulnerable dependencies
//! - `trivy_config_scan` - Scan IaC configurations for misconfigurations
//! - `trivy_sbom_generate` - Generate Software Bill of Materials
//! - `trivy_repo_scan` - Scan remote git repositories
//!
//! ## Prerequisites
//!
//! - Requires `security` feature flag
//! - Trivy CLI installed and available in PATH
//! - Appropriate permissions for scanning targets
//!
//! ## Usage
//!
//! All tools execute Trivy CLI commands and parse JSON output for structured results.

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use tracing::debug;

use super::common::{create_schema, execute_command, tool_config_with_timeout};

/// Collection of all Trivy tools
pub struct TrivyTools;

impl TrivyTools {
    /// Get all Trivy tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(TrivyImageScanTool::new()),
            Box::new(TrivyFsScanTool::new()),
            Box::new(TrivyConfigScanTool::new()),
            Box::new(TrivySbomGenerateTool::new()),
            Box::new(TrivyRepoScanTool::new()),
        ]
    }
}

/// Parse Trivy JSON output and extract vulnerability summary
fn parse_trivy_output(stdout: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(stdout).map_err(|e| format!("Failed to parse Trivy JSON output: {}", e))
}

/// Build severity filter arguments
fn build_severity_args(severity: Option<&str>) -> Vec<String> {
    if let Some(sev) = severity {
        vec!["--severity".to_string(), sev.to_uppercase()]
    } else {
        vec![]
    }
}

/// Calculate vulnerability summary from Trivy results
fn calculate_summary(results: &serde_json::Value) -> serde_json::Value {
    let mut summary = serde_json::json!({
        "total": 0,
        "critical": 0,
        "high": 0,
        "medium": 0,
        "low": 0,
        "unknown": 0
    });

    if let Some(results_array) = results.get("Results").and_then(|r| r.as_array()) {
        for result in results_array {
            if let Some(vulnerabilities) = result.get("Vulnerabilities").and_then(|v| v.as_array())
            {
                for vuln in vulnerabilities {
                    if let Some(severity) = vuln.get("Severity").and_then(|s| s.as_str()) {
                        summary["total"] = serde_json::json!(summary["total"].as_i64().unwrap_or(0) + 1);
                        match severity.to_uppercase().as_str() {
                            "CRITICAL" => {
                                summary["critical"] = serde_json::json!(
                                    summary["critical"].as_i64().unwrap_or(0) + 1
                                );
                            }
                            "HIGH" => {
                                summary["high"] =
                                    serde_json::json!(summary["high"].as_i64().unwrap_or(0) + 1);
                            }
                            "MEDIUM" => {
                                summary["medium"] = serde_json::json!(
                                    summary["medium"].as_i64().unwrap_or(0) + 1
                                );
                            }
                            "LOW" => {
                                summary["low"] =
                                    serde_json::json!(summary["low"].as_i64().unwrap_or(0) + 1);
                            }
                            _ => {
                                summary["unknown"] = serde_json::json!(
                                    summary["unknown"].as_i64().unwrap_or(0) + 1
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    summary
}

// ============================================================================
// Trivy Image Scan Tool
// ============================================================================

/// Scan a container image for vulnerabilities
pub struct TrivyImageScanTool {
    config: ToolConfig,
}

impl TrivyImageScanTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "image": {
                    "type": "string",
                    "description": "Image reference (e.g., nginx:1.25, gcr.io/project/app:v1.0)"
                },
                "severity": {
                    "type": "string",
                    "description": "Minimum severity filter: UNKNOWN, LOW, MEDIUM, HIGH, CRITICAL",
                    "enum": ["UNKNOWN", "LOW", "MEDIUM", "HIGH", "CRITICAL"]
                },
                "ignore_unfixed": {
                    "type": "boolean",
                    "description": "Skip vulnerabilities without fixes",
                    "default": false
                },
                "format": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "table", "sarif"],
                    "default": "json"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Scan timeout in seconds",
                    "default": 300
                }
            }),
            vec!["image"],
        );

        Self {
            config: tool_config_with_timeout(
                "trivy_image_scan",
                "Scan a container image for vulnerabilities using Trivy. Returns CVE details and severity counts.",
                parameters,
                300,
            ),
        }
    }
}

impl Default for TrivyImageScanTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TrivyImageScanTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let image: String = input.get_arg("image")?;
        let severity: Option<String> = input.get_arg("severity").ok();
        let ignore_unfixed: bool = input.get_arg("ignore_unfixed").unwrap_or(false);
        let format: String = input.get_arg("format").unwrap_or_else(|_| "json".to_string());
        let timeout: u64 = input.get_arg("timeout").unwrap_or(300);

        debug!(image = %image, "Scanning container image with Trivy");

        let mut args = vec!["image".to_string(), "--format".to_string(), format.clone()];

        // Add severity filter
        args.extend(build_severity_args(severity.as_deref()));

        // Add ignore_unfixed flag
        if ignore_unfixed {
            args.push("--ignore-unfixed".to_string());
        }

        // Add image reference
        args.push(image.clone());

        // Convert args to &str
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let output = match execute_command("trivy", &args_refs, None, timeout).await {
            Ok(o) => o,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to execute trivy: {}", e)));
            }
        };

        if !output.success {
            return Ok(ToolResult::error(format!(
                "Trivy scan failed (exit code {}): {}",
                output.exit_code, output.stderr
            )));
        }

        // Parse JSON output if format is json
        if format == "json" {
            let scan_results = match parse_trivy_output(&output.stdout) {
                Ok(r) => r,
                Err(e) => {
                    return Ok(ToolResult::error(e));
                }
            };

            let summary = calculate_summary(&scan_results);

            // Extract vulnerabilities list
            let mut all_vulnerabilities = vec![];
            if let Some(results_array) = scan_results.get("Results").and_then(|r| r.as_array()) {
                for result in results_array {
                    if let Some(vulnerabilities) =
                        result.get("Vulnerabilities").and_then(|v| v.as_array())
                    {
                        for vuln in vulnerabilities {
                            all_vulnerabilities.push(serde_json::json!({
                                "id": vuln.get("VulnerabilityID"),
                                "package": vuln.get("PkgName"),
                                "installed_version": vuln.get("InstalledVersion"),
                                "fixed_version": vuln.get("FixedVersion"),
                                "severity": vuln.get("Severity"),
                                "title": vuln.get("Title"),
                                "description": vuln.get("Description")
                            }));
                        }
                    }
                }
            }

            Ok(ToolResult::success(serde_json::json!({
                "success": true,
                "image": image,
                "summary": summary,
                "vulnerabilities": all_vulnerabilities,
                "raw_output": scan_results
            })))
        } else {
            // For non-JSON formats, return raw output
            Ok(ToolResult::success(serde_json::json!({
                "success": true,
                "image": image,
                "format": format,
                "output": output.stdout
            })))
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Trivy Filesystem Scan Tool
// ============================================================================

/// Scan a filesystem/directory for vulnerabilities
pub struct TrivyFsScanTool {
    config: ToolConfig,
}

impl TrivyFsScanTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Directory path to scan"
                },
                "severity": {
                    "type": "string",
                    "description": "Minimum severity filter",
                    "enum": ["UNKNOWN", "LOW", "MEDIUM", "HIGH", "CRITICAL"]
                },
                "scanners": {
                    "type": "string",
                    "description": "Comma-separated scanners: vuln, secret, misconfig",
                    "default": "vuln"
                },
                "format": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "table"],
                    "default": "json"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Scan timeout in seconds",
                    "default": 300
                }
            }),
            vec!["path"],
        );

        Self {
            config: tool_config_with_timeout(
                "trivy_fs_scan",
                "Scan a filesystem or directory for vulnerable dependencies. Supports Node.js, Python, Go, Java, Ruby projects.",
                parameters,
                300,
            ),
        }
    }
}

impl Default for TrivyFsScanTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TrivyFsScanTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path")?;
        let severity: Option<String> = input.get_arg("severity").ok();
        let scanners: String = input.get_arg("scanners").unwrap_or_else(|_| "vuln".to_string());
        let format: String = input.get_arg("format").unwrap_or_else(|_| "json".to_string());
        let timeout: u64 = input.get_arg("timeout").unwrap_or(300);

        debug!(path = %path, "Scanning filesystem with Trivy");

        let mut args = vec![
            "fs".to_string(),
            "--format".to_string(),
            format.clone(),
            "--scanners".to_string(),
            scanners,
        ];

        // Add severity filter
        args.extend(build_severity_args(severity.as_deref()));

        // Add path
        args.push(path.clone());

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let output = match execute_command("trivy", &args_refs, None, timeout).await {
            Ok(o) => o,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to execute trivy: {}", e)));
            }
        };

        if !output.success {
            return Ok(ToolResult::error(format!(
                "Trivy filesystem scan failed (exit code {}): {}",
                output.exit_code, output.stderr
            )));
        }

        if format == "json" {
            let scan_results = match parse_trivy_output(&output.stdout) {
                Ok(r) => r,
                Err(e) => {
                    return Ok(ToolResult::error(e));
                }
            };

            let summary = calculate_summary(&scan_results);

            Ok(ToolResult::success(serde_json::json!({
                "success": true,
                "path": path,
                "summary": summary,
                "results": scan_results.get("Results"),
                "raw_output": scan_results
            })))
        } else {
            Ok(ToolResult::success(serde_json::json!({
                "success": true,
                "path": path,
                "format": format,
                "output": output.stdout
            })))
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Trivy Config Scan Tool
// ============================================================================

/// Scan IaC configurations for misconfigurations
pub struct TrivyConfigScanTool {
    config: ToolConfig,
}

impl TrivyConfigScanTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Directory or file path to scan"
                },
                "config_type": {
                    "type": "string",
                    "description": "Type filter: terraform, kubernetes, dockerfile, helm",
                    "enum": ["terraform", "kubernetes", "dockerfile", "helm"]
                },
                "severity": {
                    "type": "string",
                    "description": "Minimum severity filter",
                    "enum": ["UNKNOWN", "LOW", "MEDIUM", "HIGH", "CRITICAL"]
                },
                "format": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "table"],
                    "default": "json"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Scan timeout in seconds",
                    "default": 300
                }
            }),
            vec!["path"],
        );

        Self {
            config: tool_config_with_timeout(
                "trivy_config_scan",
                "Scan IaC configurations for security misconfigurations. Supports Terraform, Kubernetes, Dockerfiles, Helm.",
                parameters,
                300,
            ),
        }
    }
}

impl Default for TrivyConfigScanTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TrivyConfigScanTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path")?;
        let config_type: Option<String> = input.get_arg("config_type").ok();
        let severity: Option<String> = input.get_arg("severity").ok();
        let format: String = input.get_arg("format").unwrap_or_else(|_| "json".to_string());
        let timeout: u64 = input.get_arg("timeout").unwrap_or(300);

        debug!(path = %path, "Scanning IaC configuration with Trivy");

        let mut args = vec![
            "config".to_string(),
            "--format".to_string(),
            format.clone(),
        ];

        // Add severity filter
        args.extend(build_severity_args(severity.as_deref()));

        // Add config type filter if specified
        if let Some(ct) = config_type {
            args.push("--file-patterns".to_string());
            args.push(format!("{}:*", ct));
        }

        // Add path
        args.push(path.clone());

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let output = match execute_command("trivy", &args_refs, None, timeout).await {
            Ok(o) => o,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to execute trivy: {}", e)));
            }
        };

        if !output.success {
            return Ok(ToolResult::error(format!(
                "Trivy config scan failed (exit code {}): {}",
                output.exit_code, output.stderr
            )));
        }

        if format == "json" {
            let scan_results = match parse_trivy_output(&output.stdout) {
                Ok(r) => r,
                Err(e) => {
                    return Ok(ToolResult::error(e));
                }
            };

            // Extract misconfigurations
            let mut all_misconfigs = vec![];
            if let Some(results_array) = scan_results.get("Results").and_then(|r| r.as_array()) {
                for result in results_array {
                    if let Some(misconfigs) =
                        result.get("Misconfigurations").and_then(|m| m.as_array())
                    {
                        for misconfig in misconfigs {
                            all_misconfigs.push(serde_json::json!({
                                "id": misconfig.get("ID"),
                                "avd_id": misconfig.get("AVDID"),
                                "title": misconfig.get("Title"),
                                "severity": misconfig.get("Severity"),
                                "description": misconfig.get("Description"),
                                "message": misconfig.get("Message"),
                                "resolution": misconfig.get("Resolution"),
                                "file": result.get("Target"),
                                "start_line": misconfig.get("CauseMetadata").and_then(|c| c.get("StartLine")),
                                "end_line": misconfig.get("CauseMetadata").and_then(|c| c.get("EndLine"))
                            }));
                        }
                    }
                }
            }

            Ok(ToolResult::success(serde_json::json!({
                "success": true,
                "path": path,
                "misconfigurations": all_misconfigs,
                "raw_output": scan_results
            })))
        } else {
            Ok(ToolResult::success(serde_json::json!({
                "success": true,
                "path": path,
                "format": format,
                "output": output.stdout
            })))
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Trivy SBOM Generate Tool
// ============================================================================

/// Generate Software Bill of Materials
pub struct TrivySbomGenerateTool {
    config: ToolConfig,
}

impl TrivySbomGenerateTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "target": {
                    "type": "string",
                    "description": "Image or directory to scan"
                },
                "format": {
                    "type": "string",
                    "description": "SBOM format",
                    "enum": ["cyclonedx", "spdx", "spdx-json"],
                    "default": "cyclonedx"
                },
                "output": {
                    "type": "string",
                    "description": "Output file path (optional, returns in response if not set)"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Scan timeout in seconds",
                    "default": 300
                }
            }),
            vec!["target", "format"],
        );

        Self {
            config: tool_config_with_timeout(
                "trivy_sbom_generate",
                "Generate Software Bill of Materials (SBOM) in CycloneDX or SPDX format for compliance and supply chain security.",
                parameters,
                300,
            ),
        }
    }
}

impl Default for TrivySbomGenerateTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TrivySbomGenerateTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let target: String = input.get_arg("target")?;
        let format: String = input.get_arg("format")?;
        let output_file: Option<String> = input.get_arg("output").ok();
        let timeout: u64 = input.get_arg("timeout").unwrap_or(300);

        debug!(target = %target, format = %format, "Generating SBOM with Trivy");

        let mut args = vec!["image".to_string(), "--format".to_string(), format.clone()];

        // Add output file if specified
        if let Some(ref out) = output_file {
            args.push("--output".to_string());
            args.push(out.clone());
        }

        // Add target
        args.push(target.clone());

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let output = match execute_command("trivy", &args_refs, None, timeout).await {
            Ok(o) => o,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to execute trivy: {}", e)));
            }
        };

        if !output.success {
            return Ok(ToolResult::error(format!(
                "Trivy SBOM generation failed (exit code {}): {}",
                output.exit_code, output.stderr
            )));
        }

        if output_file.is_some() {
            Ok(ToolResult::success(serde_json::json!({
                "success": true,
                "format": format,
                "output_file": output_file,
                "message": "SBOM generated successfully"
            })))
        } else {
            // Try to parse SBOM if JSON format
            let sbom = if format.contains("json") || format == "cyclonedx" {
                parse_trivy_output(&output.stdout).ok()
            } else {
                None
            };

            Ok(ToolResult::success(serde_json::json!({
                "success": true,
                "format": format,
                "sbom": sbom,
                "raw_output": output.stdout
            })))
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Trivy Repository Scan Tool
// ============================================================================

/// Scan a remote git repository
pub struct TrivyRepoScanTool {
    config: ToolConfig,
}

impl TrivyRepoScanTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "repo": {
                    "type": "string",
                    "description": "Repository URL (e.g., https://github.com/org/repo)"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch to scan",
                    "default": "main"
                },
                "severity": {
                    "type": "string",
                    "description": "Minimum severity filter",
                    "enum": ["UNKNOWN", "LOW", "MEDIUM", "HIGH", "CRITICAL"]
                },
                "scanners": {
                    "type": "string",
                    "description": "Comma-separated scanners: vuln, secret, misconfig",
                    "default": "vuln,secret"
                },
                "format": {
                    "type": "string",
                    "description": "Output format",
                    "enum": ["json", "table"],
                    "default": "json"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Scan timeout in seconds",
                    "default": 600
                }
            }),
            vec!["repo"],
        );

        Self {
            config: tool_config_with_timeout(
                "trivy_repo_scan",
                "Scan a remote git repository without cloning locally. Detects vulnerabilities and secrets in remote repos.",
                parameters,
                600,
            ),
        }
    }
}

impl Default for TrivyRepoScanTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TrivyRepoScanTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let repo: String = input.get_arg("repo")?;
        let branch: String = input.get_arg("branch").unwrap_or_else(|_| "main".to_string());
        let severity: Option<String> = input.get_arg("severity").ok();
        let scanners: String = input
            .get_arg("scanners")
            .unwrap_or_else(|_| "vuln,secret".to_string());
        let format: String = input.get_arg("format").unwrap_or_else(|_| "json".to_string());
        let timeout: u64 = input.get_arg("timeout").unwrap_or(600);

        debug!(repo = %repo, branch = %branch, "Scanning repository with Trivy");

        let mut args = vec![
            "repo".to_string(),
            "--format".to_string(),
            format.clone(),
            "--scanners".to_string(),
            scanners,
            "--branch".to_string(),
            branch.clone(),
        ];

        // Add severity filter
        args.extend(build_severity_args(severity.as_deref()));

        // Add repository URL
        args.push(repo.clone());

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let output = match execute_command("trivy", &args_refs, None, timeout).await {
            Ok(o) => o,
            Err(e) => {
                return Ok(ToolResult::error(format!("Failed to execute trivy: {}", e)));
            }
        };

        if !output.success {
            return Ok(ToolResult::error(format!(
                "Trivy repository scan failed (exit code {}): {}",
                output.exit_code, output.stderr
            )));
        }

        if format == "json" {
            let scan_results = match parse_trivy_output(&output.stdout) {
                Ok(r) => r,
                Err(e) => {
                    return Ok(ToolResult::error(e));
                }
            };

            let summary = calculate_summary(&scan_results);

            Ok(ToolResult::success(serde_json::json!({
                "success": true,
                "repository": repo,
                "branch": branch,
                "summary": summary,
                "results": scan_results.get("Results"),
                "raw_output": scan_results
            })))
        } else {
            Ok(ToolResult::success(serde_json::json!({
                "success": true,
                "repository": repo,
                "branch": branch,
                "format": format,
                "output": output.stdout
            })))
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trivy_tools_creation() {
        let tools = TrivyTools::all();
        assert_eq!(tools.len(), 5);

        let names: Vec<&str> = tools.iter().map(|t| t.config().name.as_str()).collect();
        assert!(names.contains(&"trivy_image_scan"));
        assert!(names.contains(&"trivy_fs_scan"));
        assert!(names.contains(&"trivy_config_scan"));
        assert!(names.contains(&"trivy_sbom_generate"));
        assert!(names.contains(&"trivy_repo_scan"));
    }

    #[test]
    fn test_image_scan_config() {
        let tool = TrivyImageScanTool::new();
        let config = tool.config();

        assert_eq!(config.name, "trivy_image_scan");
        assert!(config.description.contains("container image"));
        assert_eq!(config.timeout_secs, 300);
    }

    #[test]
    fn test_fs_scan_config() {
        let tool = TrivyFsScanTool::new();
        let config = tool.config();

        assert_eq!(config.name, "trivy_fs_scan");
        assert!(config.description.contains("filesystem"));
    }

    #[test]
    fn test_config_scan_config() {
        let tool = TrivyConfigScanTool::new();
        let config = tool.config();

        assert_eq!(config.name, "trivy_config_scan");
        assert!(config.description.contains("IaC"));
    }

    #[test]
    fn test_sbom_generate_config() {
        let tool = TrivySbomGenerateTool::new();
        let config = tool.config();

        assert_eq!(config.name, "trivy_sbom_generate");
        assert!(config.description.contains("SBOM"));
    }

    #[test]
    fn test_repo_scan_config() {
        let tool = TrivyRepoScanTool::new();
        let config = tool.config();

        assert_eq!(config.name, "trivy_repo_scan");
        assert!(config.description.contains("repository"));
        assert_eq!(config.timeout_secs, 600);
    }

    #[test]
    fn test_build_severity_args() {
        let args = build_severity_args(Some("HIGH"));
        assert_eq!(args, vec!["--severity", "HIGH"]);

        let args = build_severity_args(None);
        assert_eq!(args.len(), 0);
    }

    #[test]
    fn test_calculate_summary() {
        let sample_output = serde_json::json!({
            "Results": [
                {
                    "Vulnerabilities": [
                        { "VulnerabilityID": "CVE-2023-1", "Severity": "CRITICAL" },
                        { "VulnerabilityID": "CVE-2023-2", "Severity": "HIGH" },
                        { "VulnerabilityID": "CVE-2023-3", "Severity": "MEDIUM" },
                        { "VulnerabilityID": "CVE-2023-4", "Severity": "LOW" }
                    ]
                }
            ]
        });

        let summary = calculate_summary(&sample_output);
        assert_eq!(summary["total"].as_i64().unwrap(), 4);
        assert_eq!(summary["critical"].as_i64().unwrap(), 1);
        assert_eq!(summary["high"].as_i64().unwrap(), 1);
        assert_eq!(summary["medium"].as_i64().unwrap(), 1);
        assert_eq!(summary["low"].as_i64().unwrap(), 1);
    }
}
