---
phase: 01-onboarding-refinement
plan: 02
type: execute
wave: 2
depends_on: ["01-onboarding-refinement-01"]
files_modified:
  - aof/crates/aof-tools/src/discovery.rs
  - aof/crates/aof-tools/src/lib.rs
  - web-app/src/api/config.ts
  - web-app/src/services/toolDiscovery.ts
  - web-app/src/test/mocks/handlers.ts
  - aof/src/server/handlers/mod.rs
  - aof/src/server/handlers/tools.rs

autonomous: true

must_haves:
  truths:
    - "Tool auto-discovery scans common system paths on startup"
    - "At least 5 critical tools detected (kubectl, terraform, docker, git, aws-cli)"
    - "Tool validation checks if binary exists and can execute"
    - "Tool version is detected and displayed in UI"
    - "Discovered tools persisted and reused on daemon restart"
    - "Tools grouped by category in onboarding step"
    - "User can manually add tools via dashboard"
    - "Tool installation capability available for missing tools (future)"
    - "API endpoint `/api/tools/discover` returns list of available tools"
    - "Web UI shows tool availability status with version info"

  artifacts:
    - path: "aof/crates/aof-tools/src/discovery.rs"
      provides: "ToolDiscovery struct with scan_system_paths() method"
      exports: "ToolDiscoveryError, DiscoveredTool, ToolDiscovery impl"
    - path: "web-app/src/api/config.ts"
      provides: "configAPI.discoverTools() method calling /api/tools/discover"
      exports: "Tool type, discoverTools async function"
    - path: "web-app/src/services/toolDiscovery.ts"
      provides: "useToolDiscovery hook for tool discovery in components"
      exports: "useToolDiscovery, ToolDiscoveryState"

  key_links:
    - from: "aof-tools/discovery.rs"
      to: "server/handlers/tools.rs"
      via: "Tool handler calls ToolDiscovery::scan_system_paths()"
      pattern: "ToolDiscovery::new"
    - from: "server/handlers/tools.rs"
      to: "web-app/src/api/config.ts"
      via: "GET /api/tools/discover endpoint returns discovered tools"
      pattern: "GET /api/tools/discover"
    - from: "web-app/StepTools.tsx"
      to: "services/toolDiscovery.ts"
      via: "useToolDiscovery hook fetches and displays tools"
      pattern: "const { tools, loading } = useToolDiscovery()"

---

<objective>
Implement local tool auto-discovery that scans system paths for common DevOps tools (kubectl, terraform, docker, git, aws-cli, helm, prometheus, etc.) on daemon startup. Detected tools appear automatically in the onboarding wizard without manual configuration.

**Purpose:** Zero-friction tool setup. Users don't need to manually configure every tool; the system detects what's available and shows it.

**Output:**
- Backend ToolDiscovery service that scans /usr/bin, /usr/local/bin, ~/.local/bin, etc.
- API endpoint `/api/tools/discover` returning list of available tools with versions
- Frontend integration showing discovered tools in StepTools component
- Tool validation and version detection
- Persistent tool cache (avoids repeated scans)
</objective>

<execution_context>
@/Users/gshah/.claude/get-shit-done/workflows/execute-plan.md
@/Users/gshah/.claude/get-shit-done/templates/summary.md
@/Users/gshah/work/opsflow-sh/aof/.planning/phases/01-onboarding-refinement/RESEARCH-SUMMARY.md
</execution_context>

<context>
@/Users/gshah/work/opsflow-sh/aof/.planning/PROJECT.md
@/Users/gshah/work/opsflow-sh/aof/.planning/STATE.md
@/Users/gshah/work/opsflow-sh/aof/.planning/phases/01-onboarding-config-ui/01-INTEGRATION-SUMMARY.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Implement ToolDiscovery backend service in aof-tools crate</name>
  <files>
    aof/crates/aof-tools/src/discovery.rs
    aof/crates/aof-tools/src/lib.rs
  </files>
  <action>
Create a robust tool discovery service in the aof-tools crate:

1. **ToolDiscovery struct:**
   ```rust
   pub struct ToolDiscovery {
       cache: Arc<RwLock<HashMap<String, DiscoveredTool>>>,
       scan_paths: Vec<PathBuf>,
       last_scan: Arc<RwLock<Option<SystemTime>>>,
   }

   pub struct DiscoveredTool {
       pub id: String,              // "kubectl", "terraform", etc.
       pub name: String,            // "Kubernetes CLI"
       pub path: PathBuf,           // "/usr/local/bin/kubectl"
       pub version: Option<String>, // "v1.29.0"
       pub category: ToolCategory,  // enum: Kubectl, Terraform, Docker, Git, AWS, Shell, Helm, Custom
       pub available: bool,         // true if binary exists and executable
       pub size_bytes: u64,         // For sanity checks
       pub detected_at: SystemTime, // When it was found
   }

   pub enum ToolCategory {
       Kubectl,
       Terraform,
       Docker,
       Git,
       AWS,
       Shell,
       Helm,
       Prometheus,
       Custom(String),
   }
   ```

2. **Core methods:**
   - `new(search_paths: Option<Vec<PathBuf>>) -> Self` - Initialize with default or custom paths
   - `scan_system_paths() -> Result<Vec<DiscoveredTool>>` - Scan all paths and return tools
   - `scan_single_path(path: &Path) -> Result<Vec<DiscoveredTool>>` - Scan specific directory
   - `detect_tool_version(tool_path: &Path) -> Option<String>` - Run tool with --version, parse output
   - `is_executable(path: &Path) -> bool` - Check if file exists and is executable
   - `categorize_tool(name: &str) -> ToolCategory` - Map tool name to category

3. **Default search paths:**
   - Unix/Linux: `/usr/local/bin`, `/usr/bin`, `/opt/bin`, `~/.local/bin`, `~/.aof/tools`
   - macOS: Same as above, plus `/opt/homebrew/bin` (M1/M2 Macs)
   - Windows: `C:\\Program Files`, `C:\\Program Files (x86)`, `%USERPROFILE%\\AppData\\Local\\Programs`

4. **Tools to detect (in priority order):**
   - **Critical:** kubectl, terraform, docker, git, aws-cli, helm
   - **Important:** prometheus, loki, jq, yq, gcloud, az
   - **Optional:** vault, istio, argocd, curl, wget, ssh

5. **Version detection:**
   - Try common version flags: `--version`, `-v`, `version`
   - Parse semver from output (e.g., "kubectl version Client: v1.29.0" → "v1.29.0")
   - Graceful fallback: If version can't be detected, return None
   - Timeout: If tool takes >5 seconds to respond, skip version (don't hang)

6. **Caching:**
   - Cache results in memory (HashMap<tool_id, DiscoveredTool>)
   - Track last scan time, avoid re-scanning too frequently (cache valid for 1 hour)
   - Invalidate cache if file timestamps change (use stat comparison)
   - Provide `clear_cache()` method to force refresh

7. **Error handling:**
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum ToolDiscoveryError {
       #[error("IO error: {0}")]
       Io(#[from] std::io::Error),
       #[error("Path is not executable: {0}")]
       NotExecutable(PathBuf),
       #[error("Tool name empty")]
       EmptyToolName,
       #[error("Scan timeout")]
       ScanTimeout,
   }
   ```

8. **Example usage:**
   ```rust
   let discovery = ToolDiscovery::new(None); // Uses default paths
   let tools = discovery.scan_system_paths()?;
   for tool in tools {
       println!("{}: {} ({})", tool.name, tool.path.display(),
                tool.version.as_deref().unwrap_or("unknown"));
   }
   ```

Make discovery non-blocking: scan in background task, return cached results immediately. Use tokio::spawn for I/O operations.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof && cargo test --lib tool_discovery
Should show:
- ToolDiscovery::new() creates instance
- scan_system_paths() returns Vec of DiscoveredTool
- At least kubectl, terraform, docker detected (if installed)
- Version detection works (version field populated)
- Category assignment correct (kubectl → Kubectl category)
- Cache returns same results on second call (without re-scanning)
- is_executable() correctly identifies executable files
- Error handling works (invalid path returns error)

Manual testing:
cd /Users/gshah/work/opsflow-sh/aof && cargo build
Should compile without errors.
  </verify>
  <done>
✅ ToolDiscovery struct implemented with scan_system_paths() method
✅ Detects >=5 critical tools (kubectl, terraform, docker, git, aws-cli)
✅ Version detection works with timeout protection
✅ Caching implemented (1-hour validity, manual invalidation)
✅ Tool categories properly assigned
✅ Error handling comprehensive with user-friendly messages
✅ All tests passing (unit tests for discovery logic)
✅ No panics or unwraps in critical paths
  </done>
</task>

<task type="auto">
  <name>Task 2: Create API handler for tool discovery endpoint</name>
  <files>
    aof/src/server/handlers/tools.rs
    aof/src/server/handlers/mod.rs
  </files>
  <action>
Create HTTP handlers for tool discovery in the aofctl serve daemon:

1. **Create new handler file `server/handlers/tools.rs`:**
   ```rust
   use axum::{
       extract::State,
       http::StatusCode,
       response::IntoResponse,
       routing::get,
       Router, Json,
   };
   use serde::{Serialize, Deserialize};

   #[derive(Debug, Serialize, Deserialize, Clone)]
   pub struct ToolResponse {
       pub id: String,
       pub name: String,
       pub path: String,
       pub version: Option<String>,
       pub category: String,
       pub available: bool,
   }

   pub async fn discover_tools(
       State(tools): State<ToolDiscovery>,
   ) -> Result<impl IntoResponse, (StatusCode, String)> {
       let discovered = tools.scan_system_paths()
           .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

       let response: Vec<ToolResponse> = discovered
           .into_iter()
           .map(|tool| ToolResponse {
               id: tool.id,
               name: tool.name,
               path: tool.path.to_string_lossy().to_string(),
               version: tool.version,
               category: format!("{:?}", tool.category),
               available: tool.available,
           })
           .collect();

       Ok(Json(response))
   }

   pub fn tools_routes() -> Router<State<ToolDiscovery>> {
       Router::new()
           .route("/discover", get(discover_tools))
   }
   ```

2. **Update `server/handlers/mod.rs`:**
   - Import tools module
   - Export tools_routes()
   - Mount tools router in main serve handler

3. **Integrate into daemon startup:**
   - Initialize ToolDiscovery in serve command
   - Pass to Axum State
   - Start background task to refresh tool cache every 30 minutes

4. **Request/Response format:**
   - Request: GET /api/tools/discover (no query params)
   - Response 200:
     ```json
     [
       {
         "id": "kubectl",
         "name": "Kubernetes CLI",
         "path": "/usr/local/bin/kubectl",
         "version": "v1.29.0",
         "category": "Kubectl",
         "available": true
       },
       ...
     ]
     ```
   - Response 500: If discovery fails
     ```json
     {
       "error": "Tool discovery failed: IO error"
     }
     ```

5. **Performance considerations:**
   - Cache results to avoid repeated system scans
   - Return cached results if scanned within last hour
   - Background refresh every 30 min (not on each request)
   - Add logging: "Discovered X tools in Yms"

6. **Testing:**
   - Mock ToolDiscovery for unit tests
   - Test successful discovery (returns 200 with tools)
   - Test discovery failure (returns 500 with error)
   - Test caching (second call returns same results)

Example integration in serve.rs:
```rust
let tool_discovery = ToolDiscovery::new(None);
let tools_state = tool_discovery.clone();

// Mount tools router
let app = Router::new()
    .nest("/api/tools", tools_routes())
    .with_state(tools_state)
    // ... other routes

// Background refresh
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(30 * 60)).await;
        let _ = tool_discovery.scan_system_paths();
    }
});
```
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof && cargo test --lib tool_handler
Should show:
- Handler test: GET /api/tools/discover returns 200
- Response contains tools array
- Each tool has id, name, path, version, category, available fields
- Error case: discovery failure returns 500
- Tools properly serialized as JSON

Also start daemon and test manually:
cargo run --bin aofctl -- serve
curl http://localhost:7777/api/tools/discover
Should return JSON array of discovered tools.
  </verify>
  <done>
✅ HTTP handler GET /api/tools/discover implemented
✅ Returns JSON array of discovered tools with correct schema
✅ Error handling returns 500 with error message
✅ Response includes: id, name, path, version, category, available
✅ Handler integrated into daemon startup
✅ Background cache refresh every 30 min
✅ Tests passing (unit tests for handler)
  </done>
</task>

<task type="auto">
  <name>Task 3: Create API client method for tool discovery in web-app</name>
  <files>
    web-app/src/api/config.ts
  </files>
  <action>
Add tool discovery API client method to existing config API:

1. **Update config.ts:**
   ```typescript
   // Add to existing Tool type (if not already present)
   export interface Tool {
     id: string
     name: string
     path: string
     version: string | null
     category: string // "Kubectl", "Terraform", "Docker", "Git", "AWS", "Shell", "Helm", etc.
     available: boolean
   }

   // Add to ConfigAPI class
   export class ConfigAPI {
     // ... existing methods ...

     async discoverTools(): Promise<Tool[]> {
       try {
         const response = await this.client.get<Tool[]>('/config/tools/discover')
         return response.data
       } catch (error) {
         if (axios.isAxiosError(error)) {
           if (error.response?.status === 500) {
             throw new Error('Tool discovery failed: ' + error.response.data?.error)
           }
           throw new Error('Failed to discover tools: ' + error.message)
         }
         throw error
       }
     }

     // Optional: Get tools by category for filtering
     async getToolsByCategory(category: string): Promise<Tool[]> {
       const all = await this.discoverTools()
       return all.filter(t => t.category === category)
     }

     // Optional: Check if specific tool is available
     async isToolAvailable(toolId: string): Promise<boolean> {
       const tools = await this.discoverTools()
       return tools.some(t => t.id === toolId && t.available)
     }
   }

   // Export singleton instance
   export const configAPI = new ConfigAPI()
   ```

2. **Add mock handler for testing:**
   - Update `web-app/src/test/mocks/handlers.ts`
   - Add handler for GET /config/tools/discover
   - Return mock tools data (5-10 tools with realistic data)

3. **Export Tool type:**
   - Make Tool interface available for component imports
   - Ensure TypeScript types match backend response

4. **Error handling:**
   - Handle network errors (no backend running)
   - Handle invalid response format
   - Return empty array on error (graceful degradation)
   - Log errors for debugging

Example mock handler:
```typescript
import { http, HttpResponse } from 'msw'

const mockTools = [
  {
    id: 'kubectl',
    name: 'Kubernetes CLI',
    path: '/usr/local/bin/kubectl',
    version: 'v1.29.0',
    category: 'Kubectl',
    available: true,
  },
  {
    id: 'terraform',
    name: 'Infrastructure as Code',
    path: '/usr/local/bin/terraform',
    version: '1.6.0',
    category: 'Terraform',
    available: true,
  },
  // ... more tools
]

export const toolHandlers = [
  http.get('/config/tools/discover', () => {
    return HttpResponse.json(mockTools)
  }),
]
```
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- config.test
Should show:
- configAPI.discoverTools() callable
- Returns Promise<Tool[]>
- Mock handler returns tool data
- Error handling works

Also verify in StepTools component:
- useEffect calls configAPI.discoverTools()
- Tools populate correctly
- Type checking passes (no TypeScript errors)
  </verify>
  <done>
✅ configAPI.discoverTools() method implemented
✅ Tool interface properly typed with all required fields
✅ API client handles errors gracefully (returns error message)
✅ Mock handler in MSW returns realistic tool data
✅ TypeScript types properly exported
✅ Tests passing (API client unit tests)
  </done>
</task>

<task type="auto">
  <name>Task 4: Create useToolDiscovery React hook for components</name>
  <files>
    web-app/src/services/toolDiscovery.ts
  </files>
  <action>
Create a custom React hook for tool discovery that can be reused across components:

1. **Create toolDiscovery.ts service:**
   ```typescript
   import { useEffect, useState, useCallback } from 'react'
   import { configAPI, Tool } from '@/api/config'

   export interface ToolDiscoveryState {
     tools: Tool[]
     loading: boolean
     error: string | null
     lastDiscoveryTime: Date | null
     retrying: boolean
     discover: () => Promise<void>
   }

   export const useToolDiscovery = (): ToolDiscoveryState => {
     const [tools, setTools] = useState<Tool[]>([])
     const [loading, setLoading] = useState(false)
     const [error, setError] = useState<string | null>(null)
     const [lastDiscoveryTime, setLastDiscoveryTime] = useState<Date | null>(null)
     const [retrying, setRetrying] = useState(false)

     const discover = useCallback(async () => {
       if (loading) return // Prevent concurrent requests

       setLoading(true)
       setError(null)

       try {
         const discoveredTools = await configAPI.discoverTools()
         setTools(discoveredTools)
         setLastDiscoveryTime(new Date())

         // Log discovery summary
         const availableCount = discoveredTools.filter(t => t.available).length
         console.log(
           `Discovered ${availableCount}/${discoveredTools.length} tools in ${Date.now() - startTime}ms`
         )
       } catch (err) {
         const message = err instanceof Error ? err.message : 'Unknown error'
         setError(message)
         console.error('Tool discovery failed:', message)
       } finally {
         setLoading(false)
         setRetrying(false)
       }
     }, [loading])

     // Discover tools on mount
     useEffect(() => {
       discover()
     }, [discover])

     return {
       tools,
       loading,
       error,
       lastDiscoveryTime,
       retrying,
       discover,
     }
   }

   // Utility function to group tools by category
   export const groupToolsByCategory = (tools: Tool[]) => {
     const grouped: Record<string, Tool[]> = {}
     tools.forEach(tool => {
       if (!grouped[tool.category]) {
         grouped[tool.category] = []
       }
       grouped[tool.category].push(tool)
     })
     return grouped
   }

   // Filter tools by search query
   export const filterTools = (tools: Tool[], query: string): Tool[] => {
     const q = query.toLowerCase()
     return tools.filter(
       t => t.id.includes(q) || t.name.toLowerCase().includes(q)
     )
   }

   // Get recommended tools for initial selection
   export const getRecommendedTools = (tools: Tool[]): Tool[] => {
     const recommended = ['kubectl', 'terraform', 'docker', 'git', 'aws-cli', 'helm']
     return tools.filter(t => recommended.includes(t.id) && t.available)
   }
   ```

2. **Export utilities:**
   - `useToolDiscovery` hook for components
   - `groupToolsByCategory` for displaying tools in categories
   - `filterTools` for search functionality
   - `getRecommendedTools` for pre-selection

3. **Usage in components:**
   ```typescript
   // In StepTools component
   const { tools, loading, error, discover } = useToolDiscovery()

   if (loading) return <LoadingSpinner />
   if (error) return <AlertError message={error} onRetry={discover} />

   const grouped = groupToolsByCategory(tools)
   const recommended = getRecommendedTools(tools)
   ```

4. **Testing:**
   - Create toolDiscovery.test.ts
   - Test hook initialization
   - Test tool discovery trigger
   - Test grouping utilities
   - Test filtering utilities
   - Test error handling

Make sure hook is lightweight and doesn't cause unnecessary re-renders. Use useCallback to memoize discover function.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- toolDiscovery
Should show:
- useToolDiscovery hook loads on component mount
- Returns tools array
- Loading state updates correctly
- Error state handles failures
- discover() function callable
- groupToolsByCategory() groups tools correctly
- filterTools() filters by search query
- getRecommendedTools() returns recommended tools

Also test in components:
- StepTools uses hook correctly
- Tools display in categories
- Loading/error states visible
  </verify>
  <done>
✅ useToolDiscovery hook implemented with proper state management
✅ Hook fetches tools on mount via configAPI.discoverTools()
✅ Returns: tools, loading, error, lastDiscoveryTime, retrying, discover function
✅ Utility functions implemented: groupToolsByCategory, filterTools, getRecommendedTools
✅ Hook prevents concurrent requests (loading guard)
✅ Error handling with user-friendly messages
✅ Tests passing (hook unit tests)
  </done>
</task>

<task type="auto">
  <name>Task 5: Update MSW mock handlers for tool discovery endpoint</name>
  <files>
    web-app/src/test/mocks/handlers.ts
  </files>
  <action>
Add realistic mock tool discovery responses to MSW handlers:

1. **Create mock tools array with realistic data:**
   ```typescript
   const MOCK_TOOLS: Tool[] = [
     // Critical tools (always included)
     {
       id: 'kubectl',
       name: 'Kubernetes CLI',
       path: '/usr/local/bin/kubectl',
       version: 'v1.29.0',
       category: 'Kubectl',
       available: true,
     },
     {
       id: 'terraform',
       name: 'Infrastructure as Code',
       path: '/usr/local/bin/terraform',
       version: '1.6.0',
       category: 'Terraform',
       available: true,
     },
     {
       id: 'docker',
       name: 'Docker Container Platform',
       path: '/usr/bin/docker',
       version: '25.0.0',
       category: 'Docker',
       available: true,
     },
     {
       id: 'git',
       name: 'Version Control',
       path: '/usr/bin/git',
       version: '2.43.0',
       category: 'Git',
       available: true,
     },
     {
       id: 'aws-cli',
       name: 'AWS Command Line Interface',
       path: '/usr/local/bin/aws',
       version: '2.13.30',
       category: 'AWS',
       available: true,
     },
     {
       id: 'helm',
       name: 'Kubernetes Package Manager',
       path: '/usr/local/bin/helm',
       version: '3.13.0',
       category: 'Helm',
       available: true,
     },
     // Optional tools (may or may not be available)
     {
       id: 'prometheus',
       name: 'Prometheus Monitoring',
       path: '/usr/local/bin/prometheus',
       version: null,
       category: 'Prometheus',
       available: false, // Not installed
     },
     {
       id: 'vault',
       name: 'HashiCorp Vault',
       path: null,
       version: null,
       category: 'Custom',
       available: false,
     },
   ]
   ```

2. **Add handler for tool discovery:**
   ```typescript
   import { http, HttpResponse } from 'msw'

   export const toolHandlers = [
     http.get('*/config/tools/discover', () => {
       return HttpResponse.json(MOCK_TOOLS)
     }),
   ]
   ```

3. **Add to handlers export:**
   - Update handlers.ts to include toolHandlers
   - Make sure MSW server in test/setup.ts includes all handlers

4. **Add test scenarios:**
   - Successful discovery: returns all tools
   - Empty discovery: returns empty array (edge case)
   - Network error: return error response (optional test)

5. **Document mock data:**
   - Add comment explaining tools and their status
   - Note which tools are "always available" vs "optional"
   - Help future developers understand mock data

Example handler setup:
```typescript
export const handlers = [
  // Config endpoints
  http.get('*/config/agents', () => { ... }),
  http.post('*/config/agents', () => { ... }),

  // Tool endpoints
  http.get('*/config/tools/discover', () => {
    return HttpResponse.json(MOCK_TOOLS)
  }),

  // ... other handlers
]
```
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- mocks
Should show:
- Tool handlers properly configured
- GET /config/tools/discover returns mock tools
- Tools array properly formatted with all required fields
- Mock tools include mix of available and unavailable tools

Manual test:
- Run pnpm dev
- In browser console, call: fetch('/config/tools/discover').then(r => r.json()).then(console.log)
- Should see mock tools returned (when MSW is active in dev)
  </verify>
  <done>
✅ MSW handlers updated with tool discovery endpoint
✅ Mock tools array includes >=5 critical tools
✅ Mock includes both available and unavailable tools
✅ Each tool has proper structure (id, name, path, version, category, available)
✅ Handler returns proper JSON response
✅ Tests can verify mock data format
  </done>
</task>

<task type="auto">
  <name>Task 6: Update StepTools component to use auto-discovery</name>
  <files>
    web-app/src/components/onboarding/StepTools.tsx
  </files>
  <action>
Enhance StepTools component (from Plan 01) to properly integrate tool auto-discovery:

1. **Update component to use useToolDiscovery hook:**
   ```typescript
   import { useToolDiscovery, groupToolsByCategory, getRecommendedTools } from '@/services/toolDiscovery'

   export const StepTools: React.FC = () => {
     const dispatch = useAppDispatch()
     const selectedTools = useAppSelector(s => s.onboarding.selectedTools)

     // Use the discovery hook
     const { tools, loading, error, discover } = useToolDiscovery()

     const handleToolToggle = (tool: Tool) => {
       const isSelected = selectedTools.some(t => t.id === tool.id)
       const updated = isSelected
         ? selectedTools.filter(t => t.id !== tool.id)
         : [...selectedTools, tool]
       dispatch(updateTools(updated))
     }

     const handleRetryDiscovery = () => {
       discover()
     }

     // Group tools by category for display
     const groupedTools = groupToolsByCategory(tools)
     const recommendedTools = getRecommendedTools(tools)

     return (
       <div className="step-tools">
         <h2>Which tools can Xops use?</h2>
         <p>Select the tools you want Xops to have access to. We found these on your system.</p>

         {loading && (
           <div className="loading-state">
             <LoadingSpinner message="Scanning for available tools..." />
             <p className="text-sm text-gray-600">This may take a few seconds...</p>
           </div>
         )}

         {error && (
           <AlertWarning
             title="Tool discovery failed"
             message={error}
             onDismiss={() => {}}
             actions={[
               { label: 'Retry', onClick: handleRetryDiscovery }
             ]}
           />
         )}

         {!loading && tools.length > 0 && (
           <div className="tools-selection space-y-6">
             {/* Recommended tools section at top */}
             {recommendedTools.length > 0 && (
               <ToolCategory
                 title="Recommended Tools"
                 tools={recommendedTools}
                 selectedTools={selectedTools}
                 onToggle={handleToolToggle}
                 highlighted={true}
               />
             )}

             {/* Group remaining tools by category */}
             {Object.entries(groupedTools).map(([category, categoryTools]) => (
               <ToolCategory
                 key={category}
                 title={formatCategoryName(category)}
                 tools={categoryTools}
                 selectedTools={selectedTools}
                 onToggle={handleToolToggle}
               />
             ))}
           </div>
         )}

         {!loading && tools.length === 0 && (
           <AlertInfo
             title="No tools found"
             message="We couldn't find any tools on your system. You can add tools later in the configuration dashboard."
           />
         )}

         <div className="tools-summary mt-8">
           <p className="font-semibold">Selected tools: {selectedTools.length}</p>
           {selectedTools.length === 0 && (
             <AlertInfo
               message="No tools selected. You can add them later in the configuration dashboard."
             />
           )}
           {selectedTools.length > 0 && (
             <div className="selected-tools-list mt-2">
               {selectedTools.map(tool => (
                 <span key={tool.id} className="tool-badge">
                   {tool.name} <span className="version">{tool.version}</span>
                 </span>
               ))}
             </div>
           )}
         </div>
       </div>
     )
   }
   ```

2. **Create ToolCategory sub-component:**
   ```typescript
   interface ToolCategoryProps {
     title: string
     tools: Tool[]
     selectedTools: Tool[]
     onToggle: (tool: Tool) => void
     highlighted?: boolean
   }

   const ToolCategory: React.FC<ToolCategoryProps> = ({
     title,
     tools,
     selectedTools,
     onToggle,
     highlighted = false,
   }) => {
     const [expanded, setExpanded] = useState(true)

     return (
       <div className={`tool-category ${highlighted ? 'highlighted' : ''}`}>
         <button
           onClick={() => setExpanded(!expanded)}
           className="category-header"
         >
           <ChevronIcon expanded={expanded} />
           <h3>{title}</h3>
           <span className="count">{tools.length}</span>
         </button>

         {expanded && (
           <div className="category-tools">
             {tools.map(tool => (
               <ToolItem
                 key={tool.id}
                 tool={tool}
                 selected={selectedTools.some(t => t.id === tool.id)}
                 onToggle={() => onToggle(tool)}
               />
             ))}
           </div>
         )}
       </div>
     )
   }
   ```

3. **Create ToolItem sub-component:**
   ```typescript
   interface ToolItemProps {
     tool: Tool
     selected: boolean
     onToggle: () => void
   }

   const ToolItem: React.FC<ToolItemProps> = ({ tool, selected, onToggle }) => {
     return (
       <label className={`tool-item ${selected ? 'selected' : ''}`}>
         <input
           type="checkbox"
           checked={selected}
           onChange={onToggle}
           disabled={!tool.available}
         />
         <div className="tool-info">
           <span className="tool-name">{tool.name}</span>
           {tool.version && (
             <span className="tool-version">{tool.version}</span>
           )}
           {tool.path && (
             <span className="tool-path">{tool.path}</span>
           )}
         </div>
         <span className={`tool-status ${tool.available ? 'available' : 'unavailable'}`}>
           {tool.available ? '✓ Found' : '✗ Not found'}
         </span>
       </label>
     )
   }
   ```

4. **Handle loading and error states:**
   - Show spinner while discovering tools
   - Show error message with retry button if discovery fails
   - Allow proceeding without tools (with warning)
   - Show no tools found message if discovery returns empty

5. **Styling:**
   - Use Tailwind classes for styling
   - Consistent with existing form components
   - Visual distinction: recommended tools highlighted
   - Status indicators: green checkmark for found, gray X for not found
   - Disabled state for unavailable tools (grayed out)

Ensure component properly integrates with Redux dispatch for updateTools action.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- StepTools
Should show:
- Component renders with loading state initially
- Tools populate after discovery completes
- Tools grouped by category
- Recommended tools displayed first
- Can toggle tool selection
- Redux state updates on toggle
- Error state shows with retry button
- No tools found state displayed if empty
- All tests passing

Browser test:
- pnpm dev
- Navigate to onboarding wizard step 3
- Should see loading spinner briefly
- Tools should populate from mock API
- Can toggle tools on/off
- Selected count updates
  </verify>
  <done>
✅ StepTools component updated to use useToolDiscovery hook
✅ Tools auto-discovered and grouped by category
✅ Recommended tools displayed first (highlighted)
✅ Each tool shows name, version, path, and availability status
✅ User can toggle tools on/off
✅ Redux state updates on tool selection
✅ Loading, error, and empty states all handled
✅ Retry button available on error
✅ Can proceed with no tools selected (warning shown)
  </done>
</task>

<task type="auto">
  <name>Task 7: Write tests for tool discovery backend and frontend integration</name>
  <files>
    aof/crates/aof-tools/src/discovery.rs (tests module)
    web-app/src/services/toolDiscovery.test.ts
    web-app/src/test/e2e/tool-discovery.test.tsx
  </files>
  <action>
Create comprehensive tests for tool discovery at all layers:

1. **Backend unit tests (in discovery.rs):**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use std::path::PathBuf;

       #[test]
       fn test_categorize_tool() {
           assert_eq!(ToolDiscovery::categorize_tool("kubectl"), ToolCategory::Kubectl);
           assert_eq!(ToolDiscovery::categorize_tool("terraform"), ToolCategory::Terraform);
           assert_eq!(ToolDiscovery::categorize_tool("docker"), ToolCategory::Docker);
       }

       #[test]
       fn test_is_executable() {
           // Test with known executable (e.g., /bin/sh)
           assert!(ToolDiscovery::is_executable(&PathBuf::from("/bin/sh")));
           // Test with non-executable
           assert!(!ToolDiscovery::is_executable(&PathBuf::from("/etc/passwd")));
           // Test non-existent
           assert!(!ToolDiscovery::is_executable(&PathBuf::from("/nonexistent")));
       }

       #[tokio::test]
       async fn test_detect_version_with_timeout() {
           // Should handle timeout gracefully
           let version = ToolDiscovery::detect_version(&PathBuf::from("/bin/echo"), 1).await;
           // Should either be Some or None, never panic
           assert!(version.is_ok() || version.is_err());
       }

       #[tokio::test]
       async fn test_scan_returns_cache_on_second_call() {
           let discovery = ToolDiscovery::new(None);

           // First scan
           let tools1 = discovery.scan_system_paths().await.unwrap();

           // Second scan should return cache
           let tools2 = discovery.scan_system_paths().await.unwrap();

           assert_eq!(tools1.len(), tools2.len());
           // Should return immediately (from cache)
       }
   }
   ```

2. **Frontend service tests (toolDiscovery.test.ts):**
   ```typescript
   import { renderHook, act, waitFor } from '@testing-library/react'
   import { useToolDiscovery, groupToolsByCategory, getRecommendedTools } from '@/services/toolDiscovery'
   import { configAPI } from '@/api/config'

   describe('useToolDiscovery', () => {
     it('loads tools on mount', async () => {
       const { result } = renderHook(() => useToolDiscovery())

       expect(result.current.loading).toBe(true)

       await waitFor(() => {
         expect(result.current.loading).toBe(false)
       })

       expect(result.current.tools.length).toBeGreaterThan(0)
     })

     it('handles discovery errors', async () => {
       jest.spyOn(configAPI, 'discoverTools').mockRejectedValueOnce(new Error('Network error'))

       const { result } = renderHook(() => useToolDiscovery())

       await waitFor(() => {
         expect(result.current.loading).toBe(false)
       })

       expect(result.current.error).toBe('Network error')
     })

     it('allows manual discovery retry', async () => {
       const { result } = renderHook(() => useToolDiscovery())

       await waitFor(() => {
         expect(result.current.loading).toBe(false)
       })

       act(() => {
         result.current.discover()
       })

       expect(result.current.loading).toBe(true)
     })
   })

   describe('Tool utilities', () => {
     const mockTools = [
       { id: 'kubectl', category: 'Kubectl', available: true },
       { id: 'docker', category: 'Docker', available: true },
       { id: 'vault', category: 'Custom', available: false },
     ]

     it('groups tools by category', () => {
       const grouped = groupToolsByCategory(mockTools)
       expect(Object.keys(grouped)).toHaveLength(3)
       expect(grouped['Kubectl']).toHaveLength(1)
     })

     it('returns recommended tools', () => {
       const recommended = getRecommendedTools(mockTools)
       expect(recommended.length).toBeGreaterThan(0)
       expect(recommended[0].available).toBe(true)
     })
   })
   ```

3. **Frontend E2E tests (tool-discovery.test.tsx):**
   ```typescript
   import { render, screen, waitFor } from '@testing-library/react'
   import { OnboardingWizard } from '@/components/onboarding/OnboardingWizard'
   import { Provider } from 'react-redux'
   import { store } from '@/store'

   describe('Tool Discovery E2E', () => {
     it('discovers tools in onboarding Step 3', async () => {
       render(
         <Provider store={store}>
           <OnboardingWizard />
         </Provider>
       )

       // Navigate to Step 3
       const nextButton = screen.getByRole('button', { name: /next/i })
       // ... click through steps ...

       // On Step 3, should show loading then tools
       await waitFor(() => {
         expect(screen.getByText(/scanning for available tools/i)).toBeInTheDocument()
       })

       await waitFor(() => {
         expect(screen.getByText(/kubectl/i)).toBeInTheDocument()
       })
     })

     it('allows tool selection', async () => {
       // ... render wizard at Step 3 ...

       const kubeCheckbox = screen.getByRole('checkbox', { name: /kubernetes cli/i })
       fireEvent.click(kubeCheckbox)

       expect(screen.getByText(/selected tools: 1/i)).toBeInTheDocument()
     })

     it('handles tool discovery errors gracefully', async () => {
       // Mock API failure
       jest.spyOn(configAPI, 'discoverTools').mockRejectedValueOnce(new Error('Discovery failed'))

       // ... render wizard at Step 3 ...

       await waitFor(() => {
         expect(screen.getByText(/tool discovery failed/i)).toBeInTheDocument()
         expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument()
       })
     })
   })
   ```

4. **Test coverage goals:**
   - Backend discovery: >=80% coverage
   - Frontend service: >=85% coverage
   - E2E tests: 3+ scenarios
   - Error scenarios: All major error paths covered

Make sure tests are isolated (mock API calls) and deterministic (no timing-dependent assertions).
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof && cargo test --lib tool_discovery
All backend tests should pass.

cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- toolDiscovery
All frontend service tests should pass.

cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- tool-discovery
All E2E tests should pass.

Also:
pnpm test
All tests passing (no regressions).
  </verify>
  <done>
✅ Backend unit tests for ToolDiscovery (>=80% coverage)
✅ Frontend service tests for useToolDiscovery hook (>=85% coverage)
✅ E2E tests for tool discovery in onboarding (3+ scenarios)
✅ Error handling tested (network errors, API failures)
✅ Caching behavior tested
✅ All tests passing (pnpm test → all passing)
✅ No regressions in existing tests
  </done>
</task>

</tasks>

<verification>
After completing all tasks:

1. **Backend API verification:**
   - [ ] GET /api/tools/discover returns JSON array
   - [ ] At least 5 tools detected (kubectl, terraform, docker, git, aws-cli)
   - [ ] Each tool has: id, name, path, version, category, available fields
   - [ ] Tools cached and reused on subsequent requests
   - [ ] Background refresh every 30 minutes
   - [ ] Error handling returns 500 on failure

2. **Frontend API client verification:**
   - [ ] configAPI.discoverTools() properly typed
   - [ ] Mock handlers return realistic tool data
   - [ ] Tool interface exported and used in components
   - [ ] Error handling graceful (returns error message)

3. **React hook verification:**
   - [ ] useToolDiscovery hook loads on mount
   - [ ] Returns: tools, loading, error, discover function
   - [ ] groupToolsByCategory utility works
   - [ ] filterTools utility works
   - [ ] getRecommendedTools utility works

4. **StepTools component verification:**
   - [ ] Shows loading spinner during discovery
   - [ ] Tools populate and display in categories
   - [ ] Recommended tools highlighted and pre-selected
   - [ ] User can toggle tools on/off
   - [ ] Redux state updates on selection
   - [ ] Error state with retry button
   - [ ] Can proceed with no tools

5. **Integration verification:**
   - [ ] Full wizard flow: Step 1 → Step 2 → Step 3 → Step 4
   - [ ] Tools from discovery appear in Step 3
   - [ ] Selected tools passed to Xops creation in Step 4
   - [ ] State persists across browser refresh

6. **Testing:**
   - [ ] pnpm test → all tests passing
   - [ ] pnpm type-check → 0 TypeScript errors
   - [ ] pnpm build → succeeds without errors
   - [ ] cargo test → all backend tests passing

7. **UX quality:**
   - [ ] Tool discovery completes <3 seconds
   - [ ] Loading state shows progress
   - [ ] Error messages clear and actionable
   - [ ] No console errors or warnings
   - [ ] Mobile responsive

8. **Acceptance criteria:**
   - [ ] Auto-discovery scans common paths and finds tools
   - [ ] Tools grouped by category in UI
   - [ ] Recommended tools pre-selected
   - [ ] User can enable/disable tools
   - [ ] API endpoint working and integrated
   - [ ] Frontend properly displays discovered tools
</verification>

<success_criteria>

**Plan 02 Complete When:**

1. ✅ **Backend ToolDiscovery service:** Implemented with scan_system_paths() method
2. ✅ **Tool detection:** >=5 critical tools detected (kubectl, terraform, docker, git, aws-cli)
3. ✅ **Version detection:** Tool versions extracted from --version output
4. ✅ **API endpoint:** GET /api/tools/discover returns JSON with tool list
5. ✅ **Caching:** Tools cached for 1 hour, background refresh every 30 min
6. ✅ **Frontend API client:** configAPI.discoverTools() properly typed
7. ✅ **React hook:** useToolDiscovery hook with loading/error/discovery states
8. ✅ **Utility functions:** groupToolsByCategory, filterTools, getRecommendedTools
9. ✅ **StepTools component:** Updated to use auto-discovery with category grouping
10. ✅ **Recommended tools:** Pre-selected tools shown and highlighted
11. ✅ **Error handling:** Network errors, API errors, timeout protection all handled
12. ✅ **Testing:** Backend + frontend + E2E tests all passing
13. ✅ **No regressions:** All existing tests still passing

**Verification Method:**
```bash
# Backend tests
cargo test --lib tool_discovery

# Frontend tests
pnpm test -- toolDiscovery
pnpm test -- tool-discovery

# Full test suite
pnpm test

# Type checking
pnpm type-check

# Build
pnpm build

# Manual testing
cargo run --bin aofctl -- serve
# In another terminal:
curl http://localhost:7777/api/tools/discover
pnpm dev
# Navigate to onboarding, Step 3
```

**Deliverables:**
- 7 new/updated files (discovery.rs, tools handler, API client, hook, components, tests)
- 6+ commits with atomic changes
- 50+ lines of test coverage
- Backend ToolDiscovery service with caching
- API endpoint with proper error handling
- Frontend hook and utilities for tool discovery
- Updated StepTools component with auto-discovery
- All tests passing (0 regressions)
</success_criteria>

<output>
After completion, create `.planning/phases/01-onboarding-refinement/02-SUMMARY.md` with:
- Executive summary of tool discovery implementation
- File list (7 files created/updated)
- Test results (backend + frontend + E2E all passing)
- Performance metrics (discovery time, tools detected, cache effectiveness)
- Integration verification (tools properly displayed in UI)
- Next steps (Plan 03: Specialist bot templates)
</output>
