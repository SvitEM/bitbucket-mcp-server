# Architecture

System design and technical architecture of the Bitbucket MCP Server.

## Overview

The Bitbucket MCP Server is a Rust-based implementation of the Model Context Protocol (MCP) that provides AI assistants with access to Bitbucket Server/Data Center API 1.0. The server acts as a bridge between MCP clients (like Claude Desktop) and Bitbucket Server instances.

## High-Level Architecture

```
┌─────────────────┐
│  Claude Desktop │
│   (MCP Client)  │
└────────┬────────┘
         │ stdio (JSON-RPC 2.0)
         ▼
┌─────────────────────────────────────┐
│      Bitbucket MCP Server (Rust)    │
│  ┌───────────────────────────────┐  │
│  │   MCP Protocol Handler        │  │
│  │   - Tool registration         │  │
│  │   - Request routing           │  │
│  │   - Response serialization    │  │
│  └──────────┬────────────────────┘  │
│             │                        │
│  ┌──────────▼────────────────────┐  │
│  │   Permission Checker          │  │
│  │   - READ/WRITE/DELETE gates   │  │
│  └──────────┬────────────────────┘  │
│             │                        │
│  ┌──────────▼────────────────────┐  │
│  │   Bitbucket API Client        │  │
│  │   - HTTP Basic Auth           │  │
│  │   - SSL/TLS configuration     │  │
│  │   - Request building          │  │
│  │   - Response parsing          │  │
│  └──────────┬────────────────────┘  │
└─────────────┼────────────────────────┘
              │ HTTPS
              ▼
┌─────────────────────────────────────┐
│   Bitbucket Server/Data Center      │
│   REST API 1.0                      │
└─────────────────────────────────────┘
```

## Component Details

### 1. Entry Point (`src/main.rs`)

- Initializes tokio async runtime
- Sets up stdio transport for MCP communication
- Creates and starts the MCP server
- Handles graceful shutdown

**Key responsibilities:**
- Process lifecycle management
- Transport initialization
- Error handling at the top level

### 2. MCP Protocol Handler (`src/mcp.rs`)

Implements the `ServerHandler` trait from the rmcp SDK.

**Components:**
- `BitbucketMcpServer` struct: Main server implementation
- Tool registration: Defines all 19 MCP tools with JSON schemas
- Tool execution: Routes tool calls to appropriate API methods
- Error mapping: Converts API errors to MCP error format

**Tool Categories:**
- Projects (2 tools)
- Repositories (2 tools)
- Pull Requests (8 tools)
- Branches (3 tools)
- Commits (2 tools)
- Files (2 tools)

**Key patterns:**
```rust
// Tool registration
Tool::new(name, description, schema)

// Tool execution
async fn call_tool(&self, name: &str, args: &JsonObject) -> Result<CallToolResult>

// Response format
CallToolResult {
    content: vec![Content::text(json_string)],
    ...
}
```

### 3. Configuration System (`src/config.rs`, `src/types.rs`)

**Environment Variables:**
- `BITBUCKET_BASE_URL`: Server URL (required)
- `BITBUCKET_USERNAME`: Auth username (required)
- `BITBUCKET_PASSWORD`: Auth password/token (required)
- `BITBUCKET_SSL_VERIFY`: SSL verification toggle (default: true)
- `BITBUCKET_ALLOW_READ`: Read permission (default: true)
- `BITBUCKET_ALLOW_WRITE`: Write permission (default: true)
- `BITBUCKET_ALLOW_DELETE`: Delete permission (default: true)

**Configuration Flow:**
1. Parse ENV variables at startup
2. Validate required fields and URL format
3. Create `BitbucketConfig` struct
4. Pass to API client and permission checker

### 4. Permission System (`src/permission.rs`)

Implements fine-grained access control based on operation type.

**Components:**
- `Operation` enum: Read, Write, Delete
- `PermissionChecker` struct: Validates operations against config
- `PermissionError`: Custom error type for denied operations

**Permission Mapping:**
- **READ**: list_*, get_*, pr_diff, pr_changes, compare_branches
- **WRITE**: create_*, update_*, merge_*
- **DELETE**: decline_*

**Flow:**
```rust
// Before each API call
checker.check_permission(Operation::Write)?;
// Proceed with API call
```

### 5. API Client (`src/api/mod.rs`)

Core HTTP client for Bitbucket Server API.

**Features:**
- HTTP Basic Authentication (base64 encoded)
- SSL/TLS with optional verification disable
- URL building and normalization
- Pagination support
- Error handling and mapping

**Key methods:**
```rust
impl BitbucketClient {
    pub fn new(config: &BitbucketConfig) -> Result<Self>
    fn build_url(&self, path: &str) -> String
    fn build_paginated_url(&self, path: &str, start: Option<u32>, limit: Option<u32>) -> String
    fn default_headers(&self) -> HeaderMap
}
```

### 6. API Modules (`src/api/*`)

Each module implements a category of Bitbucket API endpoints:

- **projects.rs**: Project listing and retrieval
- **repositories.rs**: Repository operations
- **pull_requests.rs**: Full PR lifecycle (8 methods)
- **branches.rs**: Branch operations and comparison
- **commits.rs**: Commit history and details
- **files.rs**: File browsing and content retrieval

**Pattern:**
```rust
impl BitbucketClient {
    pub async fn list_projects(&self, start: Option<u32>, limit: Option<u32>) 
        -> Result<PaginatedResponse<Project>, ApiError>
    
    pub async fn get_project(&self, project_key: &str) 
        -> Result<Project, ApiError>
}
```

### 7. Error Handling (`src/api/error.rs`)

Unified error type for API operations:

```rust
pub enum ApiError {
    HttpError(reqwest::Error),
    ParseError(serde_json::Error),
    AuthError(String),
    ConfigError(String),
}
```

Errors are mapped to MCP error format in the protocol handler.

## Data Flow

### Typical Request Flow

1. **Client Request**
   - Claude Desktop sends JSON-RPC request via stdin
   - Example: `{"method": "tools/call", "params": {"name": "list_repos", ...}}`

2. **MCP Protocol Layer**
   - Parse JSON-RPC request
   - Route to appropriate tool handler
   - Extract and validate parameters

3. **Permission Check**
   - Determine operation type (Read/Write/Delete)
   - Check against ENV configuration
   - Reject if permission denied

4. **API Client**
   - Build HTTP request URL
   - Add Basic Auth header
   - Configure SSL settings
   - Send request to Bitbucket Server

5. **Response Processing**
   - Parse JSON response
   - Map to Rust structs
   - Handle pagination if applicable
   - Convert errors to MCP format

6. **Client Response**
   - Serialize result to JSON
   - Wrap in MCP response format
   - Write to stdout
   - Claude Desktop receives and processes

### Error Flow

```
API Error → ApiError enum → McpError → JSON-RPC error response → Client
```

## Technology Stack

### Core Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| **rmcp** | 0.16.0 | MCP protocol implementation |
| **tokio** | 1.41 | Async runtime |
| **reqwest** | 0.12 | HTTP client |
| **serde** | 1.0 | Serialization/deserialization |
| **serde_json** | 1.0 | JSON handling |
| **base64** | 0.22 | Basic Auth encoding |
| **anyhow** | 1.0 | Error handling |

### Build Configuration

- **Rust Version**: 1.94.0
- **Crate Type**: `["cdylib", "rlib"]` (for napi-rs compatibility)
- **Optimization**: LTO enabled, symbols stripped in release builds

## npm Packaging

### Structure

```
@bitbucket-mcp/server (main package)
├── index.js (wrapper)
├── package.json
└── optionalDependencies:
    ├── @bitbucket-mcp/darwin-x64
    ├── @bitbucket-mcp/darwin-arm64
    ├── @bitbucket-mcp/linux-x64-gnu
    ├── @bitbucket-mcp/linux-arm64-gnu
    └── @bitbucket-mcp/win32-x64
```

### Wrapper Pattern

The `index.js` wrapper:
1. Detects platform
2. Locates prebuilt binary
3. Spawns Rust binary with stdio inheritance
4. Manages process lifecycle

**Benefits:**
- Zero JavaScript dependencies
- Native performance
- Cross-platform compatibility
- Simple deployment

## CI/CD Pipeline

### GitHub Actions Workflow

**Jobs:**
1. **test**: Run cargo test and cargo audit
2. **build**: Matrix build for 5 platforms
3. **publish**: Publish to npm (on release tags)

**Platform Matrix:**
- darwin-x64: macOS Intel (native build)
- darwin-arm64: macOS Apple Silicon (native build)
- linux-x64-gnu: Linux x86_64 (native build)
- linux-arm64-gnu: Linux ARM64 (cross-compilation via cross-rs)
- win32-x64: Windows x86_64 (native build)

**Security:**
- cargo audit runs on every build
- Fails CI if vulnerabilities found
- No auto-updates (manual review required)

## Security Considerations

### Authentication

- HTTP Basic Auth only (no OAuth)
- Credentials passed via ENV variables
- Never logged or exposed in errors
- Base64 encoding for HTTP headers

### SSL/TLS

- Enabled by default
- Can be disabled for self-signed certs (corporate environments)
- Warning: Only disable for trusted internal servers

### Permission Model

- Granular control via ENV variables
- Read-only mode: Set ALLOW_WRITE=false, ALLOW_DELETE=false
- Fail-safe: Defaults to all permissions enabled
- Checked before every API call

### Input Validation

- All parameters validated before API calls
- JSON schema validation via MCP protocol
- Type safety via Rust's type system
- No SQL injection risk (REST API only)

## Performance Characteristics

### Memory Usage

- Minimal: ~5-10 MB resident memory
- No caching (stateless design)
- Async I/O prevents blocking

### Latency

- Network-bound (depends on Bitbucket Server response time)
- Typical: 50-500ms per API call
- No additional overhead from MCP protocol

### Concurrency

- Single-threaded event loop (tokio)
- Handles one MCP request at a time (stdio limitation)
- Async I/O allows efficient waiting

## Testing Strategy

### Unit Tests

- 50 tests covering core logic
- Config parsing and validation
- Permission filtering
- API client helpers (auth, URL building, pagination)
- Struct serialization/deserialization

**Coverage:**
- Config: 4 tests
- Permission: 8 tests
- API client: 5 tests
- API modules: 33 tests (struct parsing, pagination)

### No Integration Tests

- Out of scope per project requirements
- Manual QA via Claude Desktop
- Real Bitbucket Server instance for validation

## Deployment

### Installation

```bash
npm install -g @bitbucket-mcp/server
```

### Configuration

Add to Claude Desktop config:
```json
{
  "mcpServers": {
    "bitbucket": {
      "command": "npx",
      "args": ["-y", "@bitbucket-mcp/server"],
      "env": { ... }
    }
  }
}
```

### Platform Support

- macOS: 10.15+ (Intel and Apple Silicon)
- Linux: glibc 2.17+ (x86_64 and ARM64)
- Windows: Windows 10+ (x86_64)

## Future Considerations

### Potential Enhancements

- Caching for frequently accessed data
- Batch operations for multiple API calls
- Webhook support for real-time updates
- Cloud API 2.0 support (separate package)
- OAuth 2.0 authentication

### Limitations

- No retry logic (fail fast)
- No circuit breaker (simple error handling)
- No request rate limiting
- Single request at a time (stdio constraint)

## References

- [MCP Specification](https://modelcontextprotocol.io/specification/2025-06-18)
- [Bitbucket Server REST API](https://docs.atlassian.com/bitbucket-server/rest/5.16.0/bitbucket-rest.html)
- [rmcp Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [napi-rs Documentation](https://napi.rs/)
