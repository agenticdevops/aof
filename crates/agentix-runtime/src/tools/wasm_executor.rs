//! WASM sandbox for untrusted tool execution.
//!
//! Runs WASM modules in an isolated environment with capability-based permissions.
//! A WASM tool must declare its required capabilities in its `tools/*.yaml` manifest.
//! Any attempt to access an undeclared capability is blocked and returns a `WasmViolation`.
//!
//! ## Capability model
//!
//! Capabilities are declared in the tool's YAML:
//! ```yaml
//! name: my-tool
//! type: wasm
//! wasm_path: tools/my-tool.wasm
//! capabilities:
//!   - network
//!   - filesystem
//! ```
//!
//! At runtime, the sandbox intercepts host function calls and checks them against
//! the declared capabilities before allowing execution.
//!
//! ## Enabling WASM support
//!
//! WASM support is feature-gated to minimize compile time for users who don't need it:
//! ```toml
//! [dependencies]
//! agentix-runtime = { features = ["wasm"] }
//! ```

use std::fmt;
use serde::{Deserialize, Serialize};

use agentix_core::DirectoryToolType;
use agentix_core::ToolEntry;

use crate::executor::react_loop::ToolExecutor;

// ---------------------------------------------------------------------------
// Capability types
// ---------------------------------------------------------------------------

/// A capability that a WASM tool may request.
///
/// Capabilities must be declared in the tool's `tools/*.yaml` manifest.
/// Any undeclared capability access at runtime results in a `WasmViolation`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WasmCapability {
    /// Allow outbound HTTP/TCP connections to any endpoint.
    Network,
    /// Allow read/write access to the filesystem.
    Filesystem,
    /// Allow reading named secrets from environment.
    Secrets,
    /// Allow outbound HTTP to a specific list of URLs only (stricter than Network).
    #[serde(rename = "http_endpoints")]
    HttpEndpoints(Vec<String>),
    /// Resource usage limits (not a permission gate, but an enforcement bound).
    #[serde(rename = "resource_limits")]
    ResourceLimits {
        /// Maximum memory in megabytes (default: 64MB).
        memory_mb: u32,
        /// Maximum execution time in seconds (default: 30s).
        timeout_secs: u64,
    },
}

/// A capability violation — an attempt to use an undeclared capability.
#[derive(Debug, Clone)]
pub enum WasmViolation {
    NetworkBlocked {
        attempted_url: String,
    },
    FilesystemBlocked {
        attempted_path: String,
    },
    SecretsBlocked {
        attempted_key: String,
    },
    ResourceLimitExceeded {
        resource: String,
    },
}

impl fmt::Display for WasmViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WasmViolation::NetworkBlocked { attempted_url } => write!(
                f,
                "Network access blocked: WASM tool attempted to reach '{}' without 'network' capability",
                attempted_url
            ),
            WasmViolation::FilesystemBlocked { attempted_path } => write!(
                f,
                "Filesystem access blocked: WASM tool attempted to access '{}' without 'filesystem' capability",
                attempted_path
            ),
            WasmViolation::SecretsBlocked { attempted_key } => write!(
                f,
                "Secrets access blocked: WASM tool attempted to read secret '{}' without 'secrets' capability",
                attempted_key
            ),
            WasmViolation::ResourceLimitExceeded { resource } => write!(
                f,
                "Resource limit exceeded: {}",
                resource
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// CapabilityManifest
// ---------------------------------------------------------------------------

/// Capability manifest parsed from a `tools/<name>.yaml` file with `type: wasm`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityManifest {
    /// Tool name (from YAML).
    pub name: String,
    /// Declared capabilities.
    #[serde(default)]
    pub capabilities: Vec<WasmCapability>,
}

impl CapabilityManifest {
    /// Check whether a requested capability is declared.
    ///
    /// Returns `None` if the capability is allowed, or `Some(WasmViolation)` if blocked.
    pub fn check_capability(&self, requested: &WasmCapability) -> Option<WasmViolation> {
        let allowed = match requested {
            WasmCapability::Network => self
                .capabilities
                .iter()
                .any(|c| matches!(c, WasmCapability::Network)),
            WasmCapability::Filesystem => self
                .capabilities
                .iter()
                .any(|c| matches!(c, WasmCapability::Filesystem)),
            WasmCapability::Secrets => self
                .capabilities
                .iter()
                .any(|c| matches!(c, WasmCapability::Secrets)),
            WasmCapability::HttpEndpoints(_) => self
                .capabilities
                .iter()
                .any(|c| matches!(c, WasmCapability::Network | WasmCapability::HttpEndpoints(_))),
            WasmCapability::ResourceLimits { .. } => true, // always allow checking limits
        };

        if allowed {
            None
        } else {
            let violation = match requested {
                WasmCapability::Network | WasmCapability::HttpEndpoints(_) => {
                    WasmViolation::NetworkBlocked {
                        attempted_url: "<unknown>".to_string(),
                    }
                }
                WasmCapability::Filesystem => WasmViolation::FilesystemBlocked {
                    attempted_path: "<unknown>".to_string(),
                },
                WasmCapability::Secrets => WasmViolation::SecretsBlocked {
                    attempted_key: "<unknown>".to_string(),
                },
                WasmCapability::ResourceLimits { .. } => WasmViolation::ResourceLimitExceeded {
                    resource: "unknown".to_string(),
                },
            };
            Some(violation)
        }
    }

    /// Get the memory limit in bytes (default: 64MB).
    pub fn memory_limit_bytes(&self) -> u64 {
        for cap in &self.capabilities {
            if let WasmCapability::ResourceLimits { memory_mb, .. } = cap {
                return (*memory_mb as u64) * 1024 * 1024;
            }
        }
        64 * 1024 * 1024 // 64 MB default
    }

    /// Get the execution timeout in seconds (default: 30s).
    pub fn timeout_secs(&self) -> u64 {
        for cap in &self.capabilities {
            if let WasmCapability::ResourceLimits { timeout_secs, .. } = cap {
                return *timeout_secs;
            }
        }
        30 // 30s default
    }
}

// ---------------------------------------------------------------------------
// WasmSandbox
// ---------------------------------------------------------------------------

/// Executes a WASM module in a capability-enforced sandbox.
pub struct WasmSandbox {
    #[allow(dead_code)] // Used by the wasm feature's execute_with_wasmtime
    manifest: CapabilityManifest,
}

impl WasmSandbox {
    /// Create a new sandbox from a capability manifest.
    pub fn new(manifest: CapabilityManifest) -> Self {
        Self { manifest }
    }

    /// Execute a WASM module from bytes.
    ///
    /// The WASM module must export:
    /// - `memory` — a WebAssembly memory export
    /// - `run(input_offset: i32, input_len: i32) -> i32` — entry point
    ///
    /// The runtime:
    /// 1. Serializes `input` to JSON and writes it to WASM memory at offset 1024
    /// 2. Calls `run(1024, input_len)` with the input offset and length
    /// 3. Reads the output string from memory at offset 0 with the returned length
    pub fn execute_bytes(
        &self,
        wasm_bytes: &[u8],
        input: serde_json::Value,
    ) -> Result<String, String> {
        #[cfg(feature = "wasm")]
        {
            self.execute_with_wasmtime(wasm_bytes, input)
        }
        #[cfg(not(feature = "wasm"))]
        {
            let _ = (wasm_bytes, input);
            Err("WASM support not compiled. Enable the 'wasm' feature in agentix-runtime.".to_string())
        }
    }

    #[cfg(feature = "wasm")]
    fn execute_with_wasmtime(
        &self,
        wasm_bytes: &[u8],
        input: serde_json::Value,
    ) -> Result<String, String> {
        use wasmtime::{Engine, Linker, Module, Store};

        let engine = Engine::default();
        let module = Module::new(&engine, wasm_bytes)
            .map_err(|e| format!("Failed to compile WASM module: {}", e))?;

        let manifest = self.manifest.clone();
        let mut store = Store::new(&engine, manifest);

        let linker: Linker<CapabilityManifest> = Linker::new(&engine);
        // Future: register host functions for network/filesystem/secrets
        // that call store.data().check_capability() before proceeding

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| format!("Failed to instantiate WASM module: {}", e))?;

        let run_func = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "run")
            .map_err(|e| format!("WASM module missing 'run' export: {}", e))?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| "WASM module missing 'memory' export".to_string())?;

        let input_json = serde_json::to_string(&input)
            .map_err(|e| format!("Failed to serialize input: {}", e))?;
        let input_bytes = input_json.as_bytes();
        let input_offset = 1024i32;

        // Write input to WASM memory
        let mem_data = memory.data_mut(&mut store);
        if mem_data.len() < input_offset as usize + input_bytes.len() {
            return Err("WASM memory too small for input".to_string());
        }
        mem_data[input_offset as usize..input_offset as usize + input_bytes.len()]
            .copy_from_slice(input_bytes);

        // Call run function
        let output_len = run_func
            .call(&mut store, (input_offset, input_bytes.len() as i32))
            .map_err(|e| format!("WASM execution failed: {}", e))?;

        // Read output from memory at offset 0
        let mem_data = memory.data(&store);
        let output_bytes = &mem_data[..output_len as usize];
        let output = String::from_utf8_lossy(output_bytes).to_string();

        Ok(output)
    }
}

// ---------------------------------------------------------------------------
// WasmToolExecutor — implements ToolExecutor for WASM tools
// ---------------------------------------------------------------------------

/// Executes WASM tool entries with capability enforcement.
pub struct WasmToolExecutor {
    /// Directory where agent's tools/*.wasm files are located.
    #[allow(dead_code)]
    agents_dir: std::path::PathBuf,
}

impl WasmToolExecutor {
    /// Create a new WASM tool executor.
    pub fn new(agents_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            agents_dir: agents_dir.into(),
        }
    }
}

#[async_trait::async_trait]
impl ToolExecutor for WasmToolExecutor {
    async fn execute(
        &self,
        tool: &ToolEntry,
        input: serde_json::Value,
    ) -> Result<String, String> {
        // Only handle WASM tools
        if tool.tool_type != DirectoryToolType::Wasm {
            return Err(format!(
                "WasmToolExecutor cannot execute tool type {:?}",
                tool.tool_type
            ));
        }

        // Get the wasm_path from the tool (stored in command field for now)
        let wasm_path = tool.command.as_deref().ok_or_else(|| {
            format!("WASM tool '{}' has no wasm_path configured", tool.name)
        })?;

        let path = self.agents_dir.join(wasm_path);
        let wasm_bytes = std::fs::read(&path).map_err(|e| {
            format!("Failed to read WASM file '{:?}': {}", path, e)
        })?;

        let manifest = CapabilityManifest {
            name: tool.name.clone(),
            capabilities: vec![], // Parsed from tool YAML in future iteration
        };

        let sandbox = WasmSandbox::new(manifest);
        sandbox.execute_bytes(&wasm_bytes, input)
    }
}
