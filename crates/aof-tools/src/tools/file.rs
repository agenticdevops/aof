//! File System Tools
//!
//! Tools for file system operations including reading, writing, listing, and searching files.
//!
//! ## Available Tools
//!
//! - `read_file` - Read contents of a file
//! - `write_file` - Write content to a file
//! - `list_directory` - List contents of a directory
//! - `search_files` - Search for files matching a pattern
//!
//! ## MCP Alternative
//!
//! If you prefer MCP, use the filesystem MCP server:
//! ```yaml
//! mcp_servers:
//!   - name: filesystem
//!     transport: stdio
//!     command: npx
//!     args: ["@modelcontextprotocol/server-filesystem", "/workspace"]
//! ```

use aof_core::{AofResult, Tool, ToolConfig, ToolInput, ToolResult};
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tracing::debug;

use super::common::{create_schema, tool_config};

/// Collection of all file tools
pub struct FileTools;

impl FileTools {
    /// Get all file tools
    pub fn all() -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(ReadFileTool::new()),
            Box::new(WriteFileTool::new()),
            Box::new(ListDirTool::new()),
            Box::new(SearchFilesTool::new()),
        ]
    }
}

// ============================================================================
// Read File Tool
// ============================================================================

/// Read the contents of a file
pub struct ReadFileTool {
    config: ToolConfig,
}

impl ReadFileTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                },
                "encoding": {
                    "type": "string",
                    "description": "File encoding (default: utf-8)",
                    "default": "utf-8"
                },
                "max_bytes": {
                    "type": "integer",
                    "description": "Maximum bytes to read (default: 1MB)",
                    "default": 1048576
                }
            }),
            vec!["path"],
        );

        Self {
            config: tool_config(
                "read_file",
                "Read the contents of a file. Returns the file content as a string.",
                parameters,
            ),
        }
    }
}

impl Default for ReadFileTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ReadFileTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path")?;
        let max_bytes: usize = input.get_arg("max_bytes").unwrap_or(1048576);

        debug!(path = %path, "Reading file");

        let path = Path::new(&path);
        if !path.exists() {
            return Ok(ToolResult::error(format!("File not found: {}", path.display())));
        }

        let mut file = match fs::File::open(path).await {
            Ok(f) => f,
            Err(e) => return Ok(ToolResult::error(format!("Failed to open file: {}", e))),
        };

        let metadata = file.metadata().await.ok();
        let file_size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

        let mut buffer = vec![0u8; max_bytes.min(file_size as usize)];
        let bytes_read = match file.read(&mut buffer).await {
            Ok(n) => n,
            Err(e) => return Ok(ToolResult::error(format!("Failed to read file: {}", e))),
        };

        buffer.truncate(bytes_read);

        let content = String::from_utf8_lossy(&buffer).to_string();
        let truncated = bytes_read < file_size as usize;

        Ok(ToolResult::success(serde_json::json!({
            "content": content,
            "path": path.display().to_string(),
            "size": file_size,
            "bytes_read": bytes_read,
            "truncated": truncated
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Write File Tool
// ============================================================================

/// Write content to a file
pub struct WriteFileTool {
    config: ToolConfig,
}

impl WriteFileTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                },
                "append": {
                    "type": "boolean",
                    "description": "Append to file instead of overwriting (default: false)",
                    "default": false
                },
                "create_dirs": {
                    "type": "boolean",
                    "description": "Create parent directories if they don't exist (default: true)",
                    "default": true
                }
            }),
            vec!["path", "content"],
        );

        Self {
            config: tool_config(
                "write_file",
                "Write content to a file. Creates the file if it doesn't exist.",
                parameters,
            ),
        }
    }
}

impl Default for WriteFileTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for WriteFileTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path")?;
        let content: String = input.get_arg("content")?;
        let append: bool = input.get_arg("append").unwrap_or(false);
        let create_dirs: bool = input.get_arg("create_dirs").unwrap_or(true);

        debug!(path = %path, append = %append, "Writing file");

        let path = Path::new(&path);

        // Create parent directories if needed
        if create_dirs {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    if let Err(e) = fs::create_dir_all(parent).await {
                        return Ok(ToolResult::error(format!("Failed to create directories: {}", e)));
                    }
                }
            }
        }

        let result = if append {
            use tokio::io::AsyncWriteExt;
            let file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .await;
            match file {
                Ok(mut f) => f.write_all(content.as_bytes()).await,
                Err(e) => return Ok(ToolResult::error(format!("Failed to open file: {}", e))),
            }
        } else {
            fs::write(path, &content).await
        };

        match result {
            Ok(_) => Ok(ToolResult::success(serde_json::json!({
                "path": path.display().to_string(),
                "bytes_written": content.len(),
                "appended": append
            }))),
            Err(e) => Ok(ToolResult::error(format!("Failed to write file: {}", e))),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// List Directory Tool
// ============================================================================

/// List contents of a directory
pub struct ListDirTool {
    config: ToolConfig,
}

impl ListDirTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "path": {
                    "type": "string",
                    "description": "Path to the directory to list"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "List recursively (default: false)",
                    "default": false
                },
                "include_hidden": {
                    "type": "boolean",
                    "description": "Include hidden files (default: false)",
                    "default": false
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum recursion depth (default: 3)",
                    "default": 3
                }
            }),
            vec!["path"],
        );

        Self {
            config: tool_config(
                "list_directory",
                "List contents of a directory. Returns file names, sizes, and types.",
                parameters,
            ),
        }
    }

    fn list_entries_sync(
        path: &Path,
        recursive: bool,
        include_hidden: bool,
        max_depth: usize,
        current_depth: usize,
    ) -> AofResult<Vec<serde_json::Value>> {
        let mut entries = Vec::new();

        let dir = match std::fs::read_dir(path) {
            Ok(d) => d,
            Err(e) => return Err(aof_core::AofError::tool(format!("Failed to read directory: {}", e))),
        };

        for entry in dir {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files if not requested
            if !include_hidden && name.starts_with('.') {
                continue;
            }

            let metadata = entry.metadata().ok();
            let file_type = if metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false) {
                "directory"
            } else if metadata.as_ref().map(|m| m.file_type().is_symlink()).unwrap_or(false) {
                "symlink"
            } else {
                "file"
            };

            let mut entry_info = serde_json::json!({
                "name": name,
                "path": entry.path().display().to_string(),
                "type": file_type,
                "size": metadata.as_ref().map(|m| m.len()).unwrap_or(0)
            });

            // Recursively list subdirectories
            if recursive && file_type == "directory" && current_depth < max_depth {
                if let Ok(children) = Self::list_entries_sync(
                    &entry.path(), true, include_hidden, max_depth, current_depth + 1
                ) {
                    entry_info["children"] = serde_json::json!(children);
                }
            }

            entries.push(entry_info);
        }

        Ok(entries)
    }
}

impl Default for ListDirTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ListDirTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let path: String = input.get_arg("path")?;
        let recursive: bool = input.get_arg("recursive").unwrap_or(false);
        let include_hidden: bool = input.get_arg("include_hidden").unwrap_or(false);
        let max_depth: usize = input.get_arg("max_depth").unwrap_or(3);

        debug!(path = %path, recursive = %recursive, "Listing directory");

        let path = Path::new(&path);
        if !path.exists() {
            return Ok(ToolResult::error(format!("Directory not found: {}", path.display())));
        }

        if !path.is_dir() {
            return Ok(ToolResult::error(format!("Not a directory: {}", path.display())));
        }

        match Self::list_entries_sync(path, recursive, include_hidden, max_depth, 0) {
            Ok(entries) => Ok(ToolResult::success(serde_json::json!({
                "path": path.display().to_string(),
                "entries": entries,
                "count": entries.len()
            }))),
            Err(e) => Ok(ToolResult::error(e.to_string())),
        }
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

// ============================================================================
// Search Files Tool
// ============================================================================

/// Search for files matching a pattern
pub struct SearchFilesTool {
    config: ToolConfig,
}

impl SearchFilesTool {
    pub fn new() -> Self {
        let parameters = create_schema(
            serde_json::json!({
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern to match (e.g., '**/*.rs', 'src/*.ts')"
                },
                "path": {
                    "type": "string",
                    "description": "Base path to search from (default: current directory)",
                    "default": "."
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results (default: 100)",
                    "default": 100
                }
            }),
            vec!["pattern"],
        );

        Self {
            config: tool_config(
                "search_files",
                "Search for files matching a glob pattern. Returns matching file paths.",
                parameters,
            ),
        }
    }
}

impl Default for SearchFilesTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SearchFilesTool {
    async fn execute(&self, input: ToolInput) -> AofResult<ToolResult> {
        let pattern: String = input.get_arg("pattern")?;
        let base_path: String = input.get_arg("path").unwrap_or_else(|_| ".".to_string());
        let max_results: usize = input.get_arg("max_results").unwrap_or(100);

        debug!(pattern = %pattern, base_path = %base_path, "Searching files");

        let full_pattern = if pattern.starts_with('/') || pattern.starts_with('.') {
            pattern
        } else {
            format!("{}/{}", base_path, pattern)
        };

        let matches: Vec<String> = glob::glob(&full_pattern)
            .map_err(|e| aof_core::AofError::tool(format!("Invalid pattern: {}", e)))?
            .filter_map(|r| r.ok())
            .take(max_results)
            .map(|p| p.display().to_string())
            .collect();

        Ok(ToolResult::success(serde_json::json!({
            "pattern": full_pattern,
            "matches": matches,
            "count": matches.len(),
            "truncated": matches.len() >= max_results
        })))
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_read_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "Hello, World!").unwrap();

        let tool = ReadFileTool::new();
        let input = ToolInput::new(serde_json::json!({
            "path": file_path.display().to_string()
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data["content"], "Hello, World!");
    }

    #[tokio::test]
    async fn test_write_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("output.txt");

        let tool = WriteFileTool::new();
        let input = ToolInput::new(serde_json::json!({
            "path": file_path.display().to_string(),
            "content": "Test content"
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(result.success);

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Test content");
    }

    #[tokio::test]
    async fn test_list_directory() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("file1.txt"), "content1").unwrap();
        std::fs::write(dir.path().join("file2.txt"), "content2").unwrap();

        let tool = ListDirTool::new();
        let input = ToolInput::new(serde_json::json!({
            "path": dir.path().display().to_string()
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data["count"], 2);
    }

    #[tokio::test]
    async fn test_search_files() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("test.rs"), "fn main() {}").unwrap();
        std::fs::write(dir.path().join("test.txt"), "hello").unwrap();

        let tool = SearchFilesTool::new();
        let input = ToolInput::new(serde_json::json!({
            "pattern": "*.rs",
            "path": dir.path().display().to_string()
        }));

        let result = tool.execute(input).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data["count"], 1);
    }
}
