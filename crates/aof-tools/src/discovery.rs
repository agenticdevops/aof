//! Tool auto-discovery service for finding installed tools on the system
//!
//! This module scans system paths for common DevOps tools (kubectl, terraform, docker, etc.)
//! and detects their versions and metadata. Results are cached for efficiency.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, Duration};
use tokio::sync::RwLock;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use tracing::debug;

/// Error type for tool discovery operations
#[derive(Debug, Error)]
pub enum ToolDiscoveryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path is not executable: {0}")]
    NotExecutable(PathBuf),

    #[error("Tool name empty")]
    EmptyToolName,

    #[error("Scan timeout")]
    ScanTimeout,

    #[error("Version detection failed for {0}")]
    VersionDetectionFailed(String),
}

/// Category of a discovered tool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    Kubectl,
    Terraform,
    Docker,
    Git,
    AWS,
    Shell,
    Helm,
    Prometheus,
    GCloud,
    Azure,
    Custom,
}

impl std::fmt::Display for ToolCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolCategory::Kubectl => write!(f, "Kubectl"),
            ToolCategory::Terraform => write!(f, "Terraform"),
            ToolCategory::Docker => write!(f, "Docker"),
            ToolCategory::Git => write!(f, "Git"),
            ToolCategory::AWS => write!(f, "AWS"),
            ToolCategory::Shell => write!(f, "Shell"),
            ToolCategory::Helm => write!(f, "Helm"),
            ToolCategory::Prometheus => write!(f, "Prometheus"),
            ToolCategory::GCloud => write!(f, "GCloud"),
            ToolCategory::Azure => write!(f, "Azure"),
            ToolCategory::Custom => write!(f, "Custom"),
        }
    }
}

/// A discovered tool on the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredTool {
    /// Tool identifier (e.g., "kubectl", "terraform")
    pub id: String,

    /// Human-friendly name
    pub name: String,

    /// Full path to the executable
    pub path: PathBuf,

    /// Detected version (if available)
    pub version: Option<String>,

    /// Tool category for grouping
    pub category: ToolCategory,

    /// Whether the tool is available (executable exists and can run)
    pub available: bool,

    /// File size in bytes
    pub size_bytes: u64,

    /// When this tool was discovered
    pub detected_at: SystemTime,
}

impl DiscoveredTool {
    /// Create a new discovered tool
    pub fn new(
        id: String,
        name: String,
        path: PathBuf,
        category: ToolCategory,
    ) -> Self {
        Self {
            id,
            name,
            path,
            version: None,
            category,
            available: true,
            size_bytes: 0,
            detected_at: SystemTime::now(),
        }
    }
}

/// Tool discovery service
pub struct ToolDiscovery {
    cache: Arc<RwLock<HashMap<String, DiscoveredTool>>>,
    scan_paths: Vec<PathBuf>,
    last_scan: Arc<RwLock<Option<SystemTime>>>,
    cache_ttl: Duration,
}

impl ToolDiscovery {
    /// Create a new tool discovery service
    pub fn new(search_paths: Option<Vec<PathBuf>>) -> Self {
        let scan_paths = search_paths.unwrap_or_else(Self::default_paths);

        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            scan_paths,
            last_scan: Arc::new(RwLock::new(None)),
            cache_ttl: Duration::from_secs(3600), // 1 hour
        }
    }

    /// Get default system paths for tool discovery
    pub fn default_paths() -> Vec<PathBuf> {
        let mut paths = vec![
            "/usr/local/bin".into(),
            "/usr/bin".into(),
            "/opt/bin".into(),
        ];

        // Add HOME/.local/bin if home is available
        if let Ok(home) = std::env::var("HOME") {
            paths.push(PathBuf::from(home).join(".local").join("bin"));
        }

        // Add Homebrew paths for macOS
        if cfg!(target_os = "macos") {
            paths.push("/opt/homebrew/bin".into());
        }

        paths
    }

    /// Scan all configured system paths for tools
    pub async fn scan_system_paths(&self) -> Result<Vec<DiscoveredTool>, ToolDiscoveryError> {
        // Check cache validity
        let last_scan = *self.last_scan.read().await;
        if let Some(last_time) = last_scan {
            if let Ok(elapsed) = last_time.elapsed() {
                if elapsed < self.cache_ttl {
                    debug!("Returning cached tools (age: {:?})", elapsed);
                    let cache = self.cache.read().await;
                    return Ok(cache.values().cloned().collect());
                }
            }
        }

        // Perform scan
        debug!("Scanning system paths for tools");
        let mut discovered = Vec::new();

        for path in &self.scan_paths {
            match self.scan_single_path(path).await {
                Ok(mut tools) => discovered.append(&mut tools),
                Err(e) => {
                    debug!("Error scanning path {}: {}", path.display(), e);
                }
            }
        }

        // Update cache
        let mut cache = self.cache.write().await;
        cache.clear();
        for tool in &discovered {
            cache.insert(tool.id.clone(), tool.clone());
        }
        *self.last_scan.write().await = Some(SystemTime::now());

        debug!("Discovered {} tools", discovered.len());
        Ok(discovered)
    }

    /// Scan a single directory for tools
    pub async fn scan_single_path(&self, path: &Path) -> Result<Vec<DiscoveredTool>, ToolDiscoveryError> {
        let mut tools = Vec::new();

        if !path.exists() {
            return Ok(tools);
        }

        // List of known tools to look for (by ID)
        let tool_names = vec![
            // Critical tools
            ("kubectl", "Kubernetes CLI"),
            ("terraform", "Infrastructure as Code"),
            ("docker", "Docker Container Platform"),
            ("git", "Version Control"),
            ("aws", "AWS Command Line Interface"),
            ("helm", "Kubernetes Package Manager"),
            // Important tools
            ("prometheus", "Prometheus Monitoring"),
            ("loki", "Loki Log Aggregation"),
            ("jq", "JSON Query Tool"),
            ("yq", "YAML Query Tool"),
            ("gcloud", "Google Cloud CLI"),
            ("az", "Azure CLI"),
            // Optional tools
            ("vault", "HashiCorp Vault"),
            ("istio", "Istio Service Mesh"),
            ("argocd", "ArgoCD GitOps"),
            ("curl", "URL Downloader"),
            ("wget", "Web Downloader"),
            ("ssh", "SSH Client"),
            ("python", "Python Interpreter"),
            ("node", "Node.js Runtime"),
            ("npm", "Node Package Manager"),
        ];

        for (tool_id, tool_name) in tool_names {
            let tool_path = path.join(tool_id);

            if Self::is_executable(&tool_path) {
                let category = Self::categorize_tool(tool_id);
                let mut tool = DiscoveredTool::new(
                    tool_id.to_string(),
                    tool_name.to_string(),
                    tool_path.clone(),
                    category,
                );

                // Detect version
                if let Ok(version) = Self::detect_version(&tool_path, 5).await {
                    tool.version = Some(version);
                }

                // Get file size
                if let Ok(metadata) = std::fs::metadata(&tool_path) {
                    tool.size_bytes = metadata.len();
                }

                tools.push(tool);
            }
        }

        Ok(tools)
    }

    /// Check if a file is executable
    pub fn is_executable(path: &Path) -> bool {
        if let Ok(metadata) = std::fs::metadata(path) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode();
                return (mode & 0o111) != 0;
            }
            #[cfg(not(unix))]
            {
                return !metadata.is_dir();
            }
        }
        false
    }

    /// Detect tool version by running it with --version flag
    pub async fn detect_version(tool_path: &Path, timeout_secs: u64) -> Result<String, ToolDiscoveryError> {
        use tokio::time::timeout;

        let tool_path = tool_path.to_path_buf();

        // Run version detection with timeout
        let result = timeout(
            Duration::from_secs(timeout_secs),
            async move {
                let output = tokio::process::Command::new(&tool_path)
                    .arg("--version")
                    .output()
                    .await;

                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        let stderr = String::from_utf8_lossy(&out.stderr);

                        // Try stdout first, then stderr
                        let output_text = if !stdout.is_empty() {
                            stdout.to_string()
                        } else {
                            stderr.to_string()
                        };

                        // Extract version number (basic semver pattern)
                        if let Some(version) = Self::extract_version(&output_text) {
                            Ok(version)
                        } else {
                            // Return first line as fallback
                            Ok(output_text.lines().next().unwrap_or("unknown").to_string())
                        }
                    }
                    Err(_) => Err(ToolDiscoveryError::VersionDetectionFailed(
                        tool_path.display().to_string(),
                    )),
                }
            }
        ).await;

        match result {
            Ok(version_result) => version_result,
            Err(_) => Err(ToolDiscoveryError::ScanTimeout),
        }
    }

    /// Extract version string from command output
    fn extract_version(output: &str) -> Option<String> {
        // Look for patterns like "v1.29.0", "1.6.0", etc.
        let version_regex = regex::Regex::new(r"v?\d+\.\d+\.\d+")
            .unwrap_or_else(|_| regex::Regex::new(r"v?\d+\.\d+").unwrap());

        if let Some(m) = version_regex.find(output) {
            Some(m.as_str().to_string())
        } else {
            // Try to extract any number pattern
            let number_regex = regex::Regex::new(r"\d+\.\d+").ok()?;
            number_regex.find(output).map(|m| m.as_str().to_string())
        }
    }

    /// Categorize a tool by its name
    pub fn categorize_tool(name: &str) -> ToolCategory {
        match name {
            "kubectl" => ToolCategory::Kubectl,
            "terraform" => ToolCategory::Terraform,
            "docker" => ToolCategory::Docker,
            "git" => ToolCategory::Git,
            "aws" => ToolCategory::AWS,
            "helm" => ToolCategory::Helm,
            "prometheus" => ToolCategory::Prometheus,
            "gcloud" => ToolCategory::GCloud,
            "az" => ToolCategory::Azure,
            "sh" | "bash" | "zsh" => ToolCategory::Shell,
            _ => ToolCategory::Custom,
        }
    }

    /// Clear the cache (force refresh on next scan)
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
        *self.last_scan.write().await = None;
    }

    /// Get cached tools without scanning
    pub async fn get_cached(&self) -> Vec<DiscoveredTool> {
        self.cache.read().await.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_tool() {
        assert_eq!(ToolDiscovery::categorize_tool("kubectl"), ToolCategory::Kubectl);
        assert_eq!(ToolDiscovery::categorize_tool("terraform"), ToolCategory::Terraform);
        assert_eq!(ToolDiscovery::categorize_tool("docker"), ToolCategory::Docker);
        assert_eq!(ToolDiscovery::categorize_tool("git"), ToolCategory::Git);
        assert_eq!(ToolDiscovery::categorize_tool("aws"), ToolCategory::AWS);
    }

    #[test]
    fn test_is_executable() {
        // /bin/sh should be executable on Unix
        #[cfg(unix)]
        {
            assert!(ToolDiscovery::is_executable(&PathBuf::from("/bin/sh")));
        }

        // Non-existent file should not be executable
        assert!(!ToolDiscovery::is_executable(&PathBuf::from("/nonexistent/path/to/tool")));
    }

    #[test]
    fn test_extract_version() {
        assert_eq!(
            ToolDiscovery::extract_version("kubectl version Client: v1.29.0"),
            Some("v1.29.0".to_string())
        );

        assert_eq!(
            ToolDiscovery::extract_version("terraform v1.6.0"),
            Some("v1.6.0".to_string())
        );

        assert_eq!(
            ToolDiscovery::extract_version("version 1.2.3"),
            Some("1.2.3".to_string())
        );
    }

    #[tokio::test]
    async fn test_discovery_new() {
        let discovery = ToolDiscovery::new(None);
        assert!(!discovery.scan_paths.is_empty());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let discovery = ToolDiscovery::new(None);
        discovery.clear_cache().await;
        let cached = discovery.get_cached().await;
        assert_eq!(cached.len(), 0);
    }

    #[tokio::test]
    async fn test_scan_single_path_nonexistent() {
        let discovery = ToolDiscovery::new(None);
        let result = discovery.scan_single_path(Path::new("/nonexistent/path")).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_scan_system_paths_succeeds() {
        let discovery = ToolDiscovery::new(None);

        // Scan should complete without error
        let tools = discovery.scan_system_paths().await;
        assert!(tools.is_ok());

        // Verify we can get cached results
        let cached = discovery.get_cached().await;
        assert!(!cached.is_empty() || tools.unwrap().is_empty());
    }
}
