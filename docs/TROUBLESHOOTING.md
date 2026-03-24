# Troubleshooting Guide

Common issues and solutions for the Bitbucket MCP Server.

## Table of Contents

- [SSL Certificate Errors](#ssl-certificate-errors)
- [Authentication Failures](#authentication-failures)
- [Permission Denied](#permission-denied)
- [Connection Issues](#connection-issues)
- [Platform-Specific Issues](#platform-specific-issues)
- [MCP Protocol Issues](#mcp-protocol-issues)
- [Performance Issues](#performance-issues)

---

## SSL Certificate Errors

### Symptom

```
Error: SSL certificate verify failed
Error: unable to get local issuer certificate
```

### Cause

Your Bitbucket Server uses a self-signed certificate or a certificate from an internal CA that's not in the system trust store.

### Solution

Disable SSL verification (only for trusted internal servers):

```json
{
  "mcpServers": {
    "bitbucket": {
      "env": {
        "BITBUCKET_SSL_VERIFY": "false"
      }
    }
  }
}
```

**⚠️ Warning:** Only disable SSL verification for trusted internal servers. This makes the connection vulnerable to man-in-the-middle attacks.

### Alternative Solution

Add your internal CA certificate to the system trust store:

**macOS:**
```bash
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain /path/to/ca-cert.crt
```

**Linux:**
```bash
sudo cp /path/to/ca-cert.crt /usr/local/share/ca-certificates/
sudo update-ca-certificates
```

**Windows:**
```powershell
Import-Certificate -FilePath "C:\path\to\ca-cert.crt" -CertStoreLocation Cert:\LocalMachine\Root
```

---

## Authentication Failures

### Symptom

```
Error: 401 Unauthorized
Error: Authentication failed
```

### Common Causes

1. **Incorrect username**
   - Use username, not email address
   - Check for typos

2. **Expired or invalid token**
   - Personal access tokens may expire
   - Token may have been revoked

3. **Insufficient permissions**
   - User account may not have access to the project/repository
   - Token may have limited scopes

### Solutions

#### Verify Credentials

1. Test credentials manually:
```bash
curl -u username:password https://bitbucket.example.com/rest/api/1.0/projects
```

2. If successful, update Claude Desktop config with the same credentials.

#### Create New Personal Access Token

1. Go to Bitbucket Server
2. Click your profile → **Manage account**
3. Select **Personal access tokens**
4. Click **Create token**
5. Set permissions:
   - **Project permissions**: Read (minimum)
   - **Repository permissions**: Read, Write (if needed)
6. Copy the token immediately (it won't be shown again)
7. Use token as password in configuration:

```json
{
  "env": {
    "BITBUCKET_USERNAME": "your-username",
    "BITBUCKET_PASSWORD": "your-token-here"
  }
}
```

#### Check User Permissions

Verify the user has access to the project/repository in Bitbucket Server:
- Project permissions: Read (minimum)
- Repository permissions: Read, Write (for PR creation)

---

## Permission Denied

### Symptom

```
Error: Permission denied for Write operation
Error: Permission denied for Delete operation
```

### Cause

The MCP server's permission filtering is blocking the operation based on ENV configuration.

### Solution

Update the `ALLOW_*` environment variables in Claude Desktop config:

```json
{
  "env": {
    "BITBUCKET_ALLOW_READ": "true",
    "BITBUCKET_ALLOW_WRITE": "true",
    "BITBUCKET_ALLOW_DELETE": "true"
  }
}
```

### Permission Mapping

| Operation | Required Permission | Tools Affected |
|-----------|-------------------|----------------|
| **READ** | `ALLOW_READ=true` | list_*, get_*, pr_diff, pr_changes, compare_branches |
| **WRITE** | `ALLOW_WRITE=true` | create_*, update_*, merge_* |
| **DELETE** | `ALLOW_DELETE=true` | decline_* |

### Read-Only Mode

For read-only access (safe mode):

```json
{
  "env": {
    "BITBUCKET_ALLOW_READ": "true",
    "BITBUCKET_ALLOW_WRITE": "false",
    "BITBUCKET_ALLOW_DELETE": "false"
  }
}
```

---

## Connection Issues

### Symptom

```
Error: Connection refused
Error: Network unreachable
Error: Timeout
```

### Common Causes

1. **Incorrect BASE_URL**
   - Missing protocol (http:// or https://)
   - Wrong hostname or port
   - Typo in URL

2. **Network issues**
   - Firewall blocking connection
   - VPN required but not connected
   - Server is down

3. **DNS issues**
   - Hostname not resolving
   - Internal DNS not accessible

### Solutions

#### Verify BASE_URL

Correct format:
```json
{
  "env": {
    "BITBUCKET_BASE_URL": "https://bitbucket.example.com"
  }
}
```

Common mistakes:
- ❌ `bitbucket.example.com` (missing protocol)
- ❌ `https://bitbucket.example.com/` (trailing slash - OK but unnecessary)
- ❌ `http://bitbucket.example.com:7990/bitbucket` (include context path if needed)

#### Test Connectivity

```bash
# Test DNS resolution
nslookup bitbucket.example.com

# Test HTTP connectivity
curl -I https://bitbucket.example.com

# Test with authentication
curl -u username:password https://bitbucket.example.com/rest/api/1.0/projects
```

#### Check Firewall

If behind a corporate firewall:
1. Verify Bitbucket Server is accessible from your network
2. Check if VPN is required
3. Contact IT if firewall rules need updating

#### Verify Server Status

1. Open Bitbucket Server URL in browser
2. Check if server is responding
3. Verify server is not in maintenance mode

---

## Platform-Specific Issues

### macOS

#### Gatekeeper Blocking Binary

**Symptom:**
```
"bitbucket-mcp" cannot be opened because the developer cannot be verified
```

**Solution:**
1. Right-click the binary in Finder
2. Select **Open**
3. Click **Open** in the dialog
4. Or disable Gatekeeper temporarily:
```bash
sudo spctl --master-disable
```

#### Permission Denied

**Symptom:**
```
Error: EACCES: permission denied
```

**Solution:**
```bash
chmod +x /path/to/bitbucket-mcp
```

### Windows

#### Antivirus Quarantine

**Symptom:**
Binary disappears or fails to run after installation.

**Solution:**
1. Add exception in Windows Defender:
   - Settings → Update & Security → Windows Security → Virus & threat protection
   - Manage settings → Add or remove exclusions
   - Add folder: `%APPDATA%\npm\node_modules\@bitbucket-mcp`

2. Or add exception in your antivirus software

#### PowerShell Execution Policy

**Symptom:**
```
cannot be loaded because running scripts is disabled
```

**Solution:**
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Linux

#### Binary Not Executable

**Symptom:**
```
Error: EACCES: permission denied
```

**Solution:**
```bash
chmod +x ~/.npm-global/lib/node_modules/@bitbucket-mcp/server/bitbucket-mcp
```

#### Missing glibc

**Symptom:**
```
Error: version 'GLIBC_2.17' not found
```

**Solution:**
Update your system or use a newer Linux distribution. The binary requires glibc 2.17 or later.

---

## MCP Protocol Issues

### Server Not Responding

**Symptom:**
Claude Desktop shows "Server not responding" or timeout errors.

**Causes:**
1. Binary crashed
2. ENV variables not set correctly
3. stdio communication broken

**Solutions:**

1. **Check Claude Desktop logs:**

**macOS:**
```bash
tail -f ~/Library/Logs/Claude/mcp*.log
```

**Windows:**
```powershell
Get-Content "$env:APPDATA\Claude\logs\mcp*.log" -Wait
```

**Linux:**
```bash
tail -f ~/.config/Claude/logs/mcp*.log
```

2. **Test binary manually:**
```bash
export BITBUCKET_BASE_URL="https://bitbucket.example.com"
export BITBUCKET_USERNAME="your-username"
export BITBUCKET_PASSWORD="your-password"

echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | npx @bitbucket-mcp/server
```

Expected output: JSON response with server info.

3. **Verify ENV variables:**
```bash
# In Claude Desktop config, ensure all required variables are set
{
  "env": {
    "BITBUCKET_BASE_URL": "...",  // Required
    "BITBUCKET_USERNAME": "...",   // Required
    "BITBUCKET_PASSWORD": "..."    // Required
  }
}
```

### Tools Not Appearing

**Symptom:**
Bitbucket tools don't show up in Claude Desktop.

**Solutions:**

1. **Restart Claude Desktop** after config changes
2. **Check config file syntax** (valid JSON)
3. **Verify server starts** without errors in logs
4. **Check MCP server list** in Claude Desktop settings

### Invalid Tool Parameters

**Symptom:**
```
Error: Invalid parameters
Error: Missing required parameter
```

**Solution:**
Check the [API Reference](API.md) for correct parameter names and types. Common mistakes:
- Using `project` instead of `projectKey`
- Using `repo` instead of `repositorySlug`
- Missing required parameters

---

## Performance Issues

### Slow Response Times

**Causes:**
1. Network latency to Bitbucket Server
2. Large repositories or result sets
3. Bitbucket Server performance

**Solutions:**

1. **Use pagination** for large result sets:
```json
{
  "name": "list_repos",
  "arguments": {
    "projectKey": "PROJ",
    "limit": 10
  }
}
```

2. **Check Bitbucket Server performance:**
   - Monitor server load
   - Check database performance
   - Review Bitbucket Server logs

3. **Network optimization:**
   - Use VPN with better routing
   - Check for network congestion
   - Consider server location

### High Memory Usage

**Symptom:**
Binary using excessive memory (>100 MB).

**Cause:**
Large API responses being held in memory.

**Solution:**
This is unusual. If it occurs:
1. Restart Claude Desktop
2. Use smaller pagination limits
3. Report issue with reproduction steps

---

## Getting Help

### Collect Diagnostic Information

When reporting issues, include:

1. **Environment:**
   - OS and version
   - Node.js version: `node --version`
   - npm version: `npm --version`
   - Package version: `npm list -g @bitbucket-mcp/server`

2. **Configuration:**
   - Claude Desktop config (redact credentials)
   - ENV variables (redact sensitive values)

3. **Logs:**
   - Claude Desktop MCP logs
   - Error messages (full text)

4. **Steps to reproduce:**
   - What you were trying to do
   - What happened vs. what you expected

### Test Bitbucket API Directly

Isolate whether the issue is with the MCP server or Bitbucket Server:

```bash
# Test authentication
curl -u username:password \
  https://bitbucket.example.com/rest/api/1.0/projects

# Test specific endpoint
curl -u username:password \
  https://bitbucket.example.com/rest/api/1.0/projects/PROJ/repos
```

If these fail, the issue is with Bitbucket Server or credentials, not the MCP server.

### Common Error Codes

| Code | Meaning | Common Cause |
|------|---------|--------------|
| 401 | Unauthorized | Invalid credentials |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource doesn't exist |
| 500 | Internal Server Error | Bitbucket Server issue |
| -32603 | Internal Error | MCP server error |
| -32602 | Invalid Params | Wrong parameter format |

---

## Additional Resources

- [API Reference](API.md) - Complete tool documentation
- [Architecture](ARCHITECTURE.md) - System design details
- [README](../README.md) - Installation and configuration
- [Bitbucket Server REST API](https://docs.atlassian.com/bitbucket-server/rest/5.16.0/bitbucket-rest.html)
- [Model Context Protocol](https://modelcontextprotocol.io/)
