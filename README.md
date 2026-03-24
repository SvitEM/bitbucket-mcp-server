# Bitbucket MCP Server - Enterprise Secure Edition

[![npm version](https://img.shields.io/npm/v/@bitbucket-mcp/server.svg)](https://www.npmjs.com/package/@bitbucket-mcp/server)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Security Audit](https://img.shields.io/badge/security-audited-green)](https://rustsec.org)
[![Zero Runtime Dependencies](https://img.shields.io/badge/dependencies-zero-blue)](https://www.npmjs.com/package/@bitbucket-mcp/server)

**Secure, self-hosted Model Context Protocol (MCP) server for Bitbucket Server/Data Center** — designed for enterprise, air-gapped, and private network environments. Built in Rust for maximum security, performance, and zero runtime dependencies.

---

## 🎯 Why This MCP Server?

| Feature | This Server | Other MCP Servers |
|---------|-------------|-------------------|
| **Deployment** | Single binary | Node.js + npm dependencies |
| **Security** | Rust memory safety | JavaScript runtime vulnerabilities |
| **Dependencies** | Zero at runtime | 100+ npm packages |
| **Air-gapped** | ✅ Fully supported | ❌ Requires npm registry |
| **SSL/TLS** | Custom CA support | Limited |
| **Permissions** | Granular READ/WRITE/DELETE | All-or-nothing |
| **Audit Trail** | Security-audited dependencies | Supply chain risks |

### Perfect For:
- 🔒 **Enterprise environments** with strict security policies
- 🏢 **Corporate networks** behind firewalls
- ✈️ **Air-gapped systems** without internet access
- 🛡️ **Compliance requirements** (SOC2, ISO27001, GDPR)
- 📦 **Minimal attack surface** requirements

---

## 🚀 Quick Start

### Install (30 seconds)

```bash
# Global install
npm install -g @bitbucket-mcp/server

# Or run without install
npx @bitbucket-mcp/server
```

### Configure Claude Desktop

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`  
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`  
**Linux**: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "bitbucket": {
      "command": "npx",
      "args": ["-y", "@bitbucket-mcp/server"],
      "env": {
        "BITBUCKET_BASE_URL": "https://bitbucket.your-company.com",
        "BITBUCKET_USERNAME": "service-account",
        "BITBUCKET_PASSWORD": "your-personal-access-token",
        "BITBUCKET_SSL_VERIFY": "true",
        "BITBUCKET_ALLOW_READ": "true",
        "BITBUCKET_ALLOW_WRITE": "false",
        "BITBUCKET_ALLOW_DELETE": "false"
      }
    }
  }
}
```

---

## 🔐 Security Features

### Enterprise-Grade Security

| Feature | Description |
|---------|-------------|
| **Memory Safety** | Rust prevents buffer overflows, use-after-free, data races |
| **Zero Supply Chain Risk** | No runtime npm dependencies to compromise |
| **Audited Dependencies** | All Rust crates scanned with `cargo-audit` |
| **Granular Permissions** | Separate READ/WRITE/DELETE controls |
| **Self-Signed SSL** | Support for internal CA certificates |
| **No Telemetry** | Zero data collection, fully offline-capable |

### Security Configuration

```json
{
  "env": {
    // Require SSL verification (default: true)
    "BITBUCKET_SSL_VERIFY": "true",
    
    // Read-only mode for maximum security
    "BITBUCKET_ALLOW_READ": "true",
    "BITBUCKET_ALLOW_WRITE": "false",
    "BITBUCKET_ALLOW_DELETE": "false",
    
    // Use personal access token instead of password
    "BITBUCKET_PASSWORD": "your-pat-token"
  }
}
```

### Compliance Ready

- ✅ **SOC2**: Audit trails, access controls, encryption in transit
- ✅ **ISO27001**: Secure development, dependency management
- ✅ **GDPR**: No data retention, on-premise deployment
- ✅ **HIPAA**: Private network deployment supported

---

## 🛠️ Available Tools (19 MCP Tools)

### Projects
| Tool | Description | Permission |
|------|-------------|------------|
| `list_projects` | List all projects with pagination | READ |
| `get_project` | Get project details by key | READ |

### Repositories
| Tool | Description | Permission |
|------|-------------|------------|
| `list_repos` | List repositories in a project | READ |
| `get_repo` | Get repository details | READ |

### Pull Requests
| Tool | Description | Permission |
|------|-------------|------------|
| `list_prs` | List pull requests with filters (state, author) | READ |
| `get_pr` | Get pull request details | READ |
| `create_pr` | Create a new pull request | WRITE |
| `update_pr` | Update pull request title/description | WRITE |
| `merge_pr` | Merge a pull request | DELETE |
| `decline_pr` | Decline a pull request | DELETE |
| `pr_diff` | Get pull request diff | READ |
| `pr_changes` | Get pull request file changes | READ |

### Branches
| Tool | Description | Permission |
|------|-------------|------------|
| `list_branches` | List repository branches | READ |
| `create_branch` | Create a new branch from ref | WRITE |
| `compare_branches` | Compare commits between branches | READ |

### Commits
| Tool | Description | Permission |
|------|-------------|------------|
| `list_commits` | List repository commits with filters | READ |
| `get_commit` | Get commit details | READ |

### Files
| Tool | Description | Permission |
|------|-------------|------------|
| `list_files` | Browse repository files | READ |
| `get_file_content` | Get file content at specific commit | READ |

---

## 📦 Deployment Options

### Option 1: NPM (Recommended)

```bash
npm install -g @bitbucket-mcp/server
```

### Option 2: Direct Binary (Air-Gapped)

Download prebuilt binary for your platform:

| Platform | Binary | Size |
|----------|--------|------|
| macOS Intel | `bitbucket-mcp-darwin-x64` | ~15 MB |
| macOS ARM | `bitbucket-mcp-darwin-arm64` | ~12 MB |
| Linux x64 | `bitbucket-mcp-linux-x64` | ~18 MB |
| Linux ARM64 | `bitbucket-mcp-linux-arm64` | ~16 MB |
| Windows x64 | `bitbucket-mcp-win32-x64.exe` | ~20 MB |

```bash
# Download and run
chmod +x bitbucket-mcp-linux-x64
./bitbucket-mcp-linux-x64
```

### Option 3: Docker (Coming Soon)

```dockerfile
FROM scratch
COPY bitbucket-mcp-linux-x64 /bitbucket-mcp
ENTRYPOINT ["/bitbucket-mcp"]
```

---

## 🏢 Enterprise Deployment

### Air-Gapped Installation

1. **Download on internet-connected machine:**
   ```bash
   npm pack @bitbucket-mcp/server
   ```

2. **Transfer to air-gapped system** via secure media

3. **Install offline:**
   ```bash
   npm install -g bitbucket-mcp-server-0.1.0.tgz
   ```

### Corporate Proxy Support

```json
{
  "env": {
    "BITBUCKET_BASE_URL": "https://bitbucket.internal.company.com",
    "HTTP_PROXY": "http://proxy.company.com:8080",
    "HTTPS_PROXY": "http://proxy.company.com:8080",
    "NO_PROXY": "localhost,127.0.0.1,.internal.company.com"
  }
}
```

### Service Account Setup

```bash
# 1. Create dedicated service account in Bitbucket
# 2. Grant minimal required permissions
# 3. Generate Personal Access Token (PAT)
# 4. Use in configuration (never commit to git!)
```

---

## 🔧 Configuration Reference

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `BITBUCKET_BASE_URL` | **Yes** | - | Bitbucket Server base URL |
| `BITBUCKET_USERNAME` | **Yes** | - | Username for Basic Auth |
| `BITBUCKET_PASSWORD` | **Yes** | - | Password or PAT |
| `BITBUCKET_SSL_VERIFY` | No | `true` | SSL certificate verification |
| `BITBUCKET_ALLOW_READ` | No | `true` | Allow read operations |
| `BITBUCKET_ALLOW_WRITE` | No | `true` | Allow write operations |
| `BITBUCKET_ALLOW_DELETE` | No | `true` | Allow delete operations |

### Permission Modes

```json
// Read-only (maximum security)
{ "BITBUCKET_ALLOW_READ": "true", "BITBUCKET_ALLOW_WRITE": "false", "BITBUCKET_ALLOW_DELETE": "false" }

// Developer mode (no delete)
{ "BITBUCKET_ALLOW_READ": "true", "BITBUCKET_ALLOW_WRITE": "true", "BITBUCKET_ALLOW_DELETE": "false" }

// Admin mode (full access)
{ "BITBUCKET_ALLOW_READ": "true", "BITBUCKET_ALLOW_WRITE": "true", "BITBUCKET_ALLOW_DELETE": "true" }
```

---

## 🛡️ Security Best Practices

### 1. Use Personal Access Tokens

```bash
# ❌ Don't use password
BITBUCKET_PASSWORD="my-actual-password"

# ✅ Use PAT with minimal scope
BITBUCKET_PASSWORD="pat:read-only-projects-repos"
```

### 2. Enable Read-Only Mode

```json
{
  "env": {
    "BITBUCKET_ALLOW_WRITE": "false",
    "BITBUCKET_ALLOW_DELETE": "false"
  }
}
```

### 3. Restrict Network Access

```bash
# Bind to localhost only (if supported)
# Or use firewall rules to restrict access
sudo ufw allow from 127.0.0.1 to any port 3000
```

### 4. Rotate Credentials

```bash
# Rotate PAT every 90 days
# Monitor Bitbucket audit logs for unusual activity
```

### 5. Enable SSL Verification

```json
{
  "env": {
    "BITBUCKET_SSL_VERIFY": "true"  // Never disable in production
  }
}
```

---

## 🐛 Troubleshooting

### SSL Certificate Errors

**For self-signed/internal CA certificates:**

```json
{
  "env": {
    // Option 1: Add to system trust store (recommended)
    // Option 2: Disable verification (NOT recommended for production)
    "BITBUCKET_SSL_VERIFY": "false"
  }
}
```

**⚠️ Warning:** Only disable SSL verification for trusted internal servers.

### Authentication Failed

```bash
# Check credentials
curl -u username:password https://bitbucket.example.com/rest/api/1.0/projects

# Verify PAT has required permissions
# Check user is not locked/suspended
```

### Connection Refused

```bash
# Verify URL includes protocol
BITBUCKET_BASE_URL="https://bitbucket.example.com"  # ✅
BITBUCKET_BASE_URL="bitbucket.example.com"          # ❌

# Test connectivity
curl -I https://bitbucket.example.com

# Check firewall/proxy rules
```

### Permission Denied

```json
{
  "env": {
    // Enable only required permissions
    "BITBUCKET_ALLOW_READ": "true",
    "BITBUCKET_ALLOW_WRITE": "false",
    "BITBUCKET_ALLOW_DELETE": "false"
  }
}
```

---

## 🏗️ Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  Claude Desktop │────▶│  MCP Server      │────▶│  Bitbucket      │
│  (AI Assistant) │     │  (Rust Binary)   │     │  Server/DC      │
└─────────────────┘     └──────────────────┘     └─────────────────┘
        │                       │                        │
        │                       │                        │
   User Requests          Zero Dependencies        REST API 1.0
   Natural Language         Memory Safe              Basic Auth
                            Single Binary            SSL/TLS
```

### Why Rust?

- **Memory Safety**: No buffer overflows, use-after-free, or data races
- **Performance**: Native code, no GC pauses
- **Security**: Audited dependencies, minimal attack surface
- **Portability**: Single static binary, no runtime dependencies

---

## 📚 Resources

- [Model Context Protocol Documentation](https://modelcontextprotocol.io/)
- [Bitbucket Server REST API](https://docs.atlassian.com/bitbucket-server/rest/5.16.0/bitbucket-rest.html)
- [Security Best Practices](docs/SECURITY.md)
- [Enterprise Deployment Guide](docs/ENTERPRISE.md)
- [Build Instructions](docs/BUILD.md)

---

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development

```bash
git clone https://github.com/your-org/bitbucket-mcp-server.git
cd bitbucket-mcp-server

# Build
cargo build --release

# Test
cargo test

# Security audit
cargo audit
```

---

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

---

## 🔍 Keywords

MCP server, Model Context Protocol, Bitbucket Server, Bitbucket Data Center, enterprise MCP, secure MCP, self-hosted MCP, air-gapped MCP, on-premise MCP, single binary MCP, Rust MCP server, zero dependencies, private network MCP, corporate MCP, SOC2 compliant, ISO27001 compliant, Atlassian integration, Bitbucket REST API, Claude Desktop MCP, AI integration

---

<p align="center">
  <strong>Built for enterprise security • Zero runtime dependencies • Air-gapped ready</strong>
</p>
