# Build Instructions for Different Platforms

This guide covers building the Bitbucket MCP Server for all supported platforms.

## Prerequisites

### Required Tools

- **Rust** (1.94.0 or later)
- **Node.js** (18.x or later) - for npm packaging
- **Git** - for version control

### Installation

```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js (macOS)
brew install node@18

# Install Node.js (Linux - Ubuntu/Debian)
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# Install Node.js (Windows)
# Download from https://nodejs.org/
```

---

## Platform-Specific Build Instructions

### macOS (Intel x64)

```bash
# Add target (if not already added)
rustup target add x86_64-apple-darwin

# Build release binary
cargo build --release --target x86_64-apple-darwin

# Binary location
ls -la target/x86_64-apple-darwin/release/bitbucket-mcp
```

### macOS (Apple Silicon ARM64)

```bash
# Add target (if not already added)
rustup target add aarch64-apple-darwin

# Build release binary
cargo build --release --target aarch64-apple-darwin

# Binary location
ls -la target/aarch64-apple-darwin/release/bitbucket-mcp
```

### Linux (x64)

```bash
# Add target (if not already added)
rustup target add x86_64-unknown-linux-gnu

# Build release binary
cargo build --release --target x86_64-unknown-linux-gnu

# Binary location
ls -la target/x86_64-unknown-linux-gnu/release/bitbucket-mcp
```

### Linux (ARM64)

**Option 1: Using cross (recommended for cross-compilation)**

```bash
# Install cross (one-time setup)
cargo install cross --git https://github.com/cross-rs/cross

# Build with cross
cross build --release --target aarch64-unknown-linux-gnu

# Binary location
ls -la target/aarch64-unknown-linux-gnu/release/bitbucket-mcp
```

**Option 2: Native build on ARM64 Linux**

```bash
# Add target
rustup target add aarch64-unknown-linux-gnu

# Build release binary
cargo build --release --target aarch64-unknown-linux-gnu
```

### Windows (x64)

```bash
# Add target (if not already added)
rustup target add x86_64-pc-windows-msvc

# Build release binary
cargo build --release --target x86_64-pc-windows-msvc

# Binary location
ls target/x86_64-pc-windows-msvc/release/bitbucket-mcp.exe
```

---

## Multi-Platform Build (All Platforms at Once)

Create a build script to compile for all platforms:

```bash
#!/bin/bash
# build-all.sh

set -e

PLATFORMS=(
  "x86_64-apple-darwin"
  "aarch64-apple-darwin"
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
  "x86_64-pc-windows-msvc"
)

echo "Installing required targets..."
for target in "${PLATFORMS[@]}"; do
  rustup target add "$target" || true
done

echo "Building for all platforms..."
for target in "${PLATFORMS[@]}"; do
  echo "Building for $target..."
  
  if [ "$target" == "aarch64-unknown-linux-gnu" ]; then
    cross build --release --target "$target"
  else
    cargo build --release --target "$target"
  fi
  
  echo "✓ Built for $target"
done

echo "All builds complete!"
```

Usage:
```bash
chmod +x build-all.sh
./build-all.sh
```

---

## Building NPM Package

The project uses a monorepo structure with platform-specific packages.

### Directory Structure

```
bitbucket-mcp/
├── package.json              # Main package
├── index.js                  # Entry point
├── Cargo.toml                # Rust dependencies
└── artifacts/
    ├── darwin-x64/           # macOS Intel
    ├── darwin-arm64/         # macOS ARM
    ├── linux-x64-gnu/        # Linux x64
    ├── linux-arm64-gnu/      # Linux ARM
    └── win32-x64/            # Windows x64
```

### Build and Package Process

```bash
# 1. Build all platform binaries
./build-all.sh

# 2. Copy binaries to artifact directories
mkdir -p artifacts/darwin-x64
mkdir -p artifacts/darwin-arm64
mkdir -p artifacts/linux-x64-gnu
mkdir -p artifacts/linux-arm64-gnu
mkdir -p artifacts/win32-x64

cp target/x86_64-apple-darwin/release/bitbucket-mcp artifacts/darwin-x64/
cp target/aarch64-apple-darwin/release/bitbucket-mcp artifacts/darwin-arm64/
cp target/x86_64-unknown-linux-gnu/release/bitbucket-mcp artifacts/linux-x64-gnu/
cp target/aarch64-unknown-linux-gnu/release/bitbucket-mcp artifacts/linux-arm64-gnu/
cp target/x86_64-pc-windows-msvc/release/bitbucket-mcp.exe artifacts/win32-x64/

# 3. Build npm package
npm run build

# 4. Test package locally
npm pack

# 5. Publish to npm (requires NPM_TOKEN)
npm publish --access public
```

---

## CI/CD Build (GitHub Actions)

The project includes automated multi-platform builds via GitHub Actions.

### Workflow: `.github/workflows/release.yml`

**Triggered by:**
- Push to `main` branch
- Pull requests
- Release publication

**Build Matrix:**
| Platform | OS | Target | Binary Name |
|----------|-----|--------|-------------|
| darwin-x64 | macos-latest | x86_64-apple-darwin | bitbucket-mcp |
| darwin-arm64 | macos-latest | aarch64-apple-darwin | bitbucket-mcp |
| linux-x64-gnu | ubuntu-latest | x86_64-unknown-linux-gnu | bitbucket-mcp |
| linux-arm64-gnu | ubuntu-latest | aarch64-unknown-linux-gnu | bitbucket-mcp |
| win32-x64 | windows-latest | x86_64-pc-windows-msvc | bitbucket-mcp.exe |

**Steps:**
1. Checkout code
2. Setup Rust 1.94.0
3. Install `cross` for ARM Linux (cross-compilation)
4. Build release binary
5. Upload artifacts

**Publish (on release only):**
1. Download all artifacts
2. Package for npm
3. Publish to npm registry

---

## Build Configuration

### Cargo.toml Settings

```toml
[lib]
crate-type = ["cdylib", "rlib"]  # C-compatible dynamic library + Rust library

[dependencies]
napi = "2.16"        # Node-API bindings
napi-derive = "2.16" # NAPI macros

[build-dependencies]
napi-build = "2.1"   # Build script helpers

[profile.release]
lto = true           # Link-time optimization
strip = true         # Strip debug symbols
```

### Build Optimizations

The release profile includes:
- **LTO (Link-Time Optimization)**: Improves performance by optimizing across crate boundaries
- **Strip**: Removes debug symbols, reducing binary size

### Custom Build Profiles

```toml
# Add to Cargo.toml for smaller binaries
[profile.release-small]
inherits = "release"
opt-level = "z"      # Optimize for size
lto = true
codegen-units = 1    # Single codegen unit for better optimization
```

---

## Testing Builds

### Run Tests

```bash
# Run all tests
cargo test

# Run tests for specific platform
cargo test --target x86_64-unknown-linux-gnu

# Run tests with output
cargo test -- --nocapture
```

### Security Audit

```bash
# Install cargo-audit (one-time)
cargo install cargo-audit

# Run security audit
cargo audit
```

### Verify Binary

```bash
# Check binary architecture (macOS/Linux)
file target/*/release/bitbucket-mcp

# Example output:
# target/x86_64-apple-darwin/release/bitbucket-mcp: Mach-O 64-bit executable x86_64
# target/aarch64-apple-darwin/release/bitbucket-mcp: Mach-O 64-bit executable arm64
# target/x86_64-unknown-linux-gnu/release/bitbucket-mcp: ELF 64-bit LSB executable, x86-64

# Check binary size
ls -lh target/*/release/bitbucket-mcp*
```

---

## Troubleshooting

### Missing Target

**Error:** `error[E0463]: can't find crate for 'std'`

**Solution:**
```bash
rustup target add <target-triple>
```

### Cross-Compilation for ARM Linux Fails

**Error:** Linker errors or missing sysroot

**Solution:** Use `cross` instead of `cargo`:
```bash
cargo install cross --git https://github.com/cross-rs/cross
cross build --release --target aarch64-unknown-linux-gnu
```

### macOS Universal Binary

To create a single binary that works on both Intel and Apple Silicon:

```bash
# Build both architectures
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Combine with lipo
mkdir -p target/universal/release
lipo -create \
  target/x86_64-apple-darwin/release/bitbucket-mcp \
  target/aarch64-apple-darwin/release/bitbucket-mcp \
  -output target/universal/release/bitbucket-mcp

# Verify
file target/universal/release/bitbucket-mcp
# Output: Mach-O universal binary with 2 architectures
```

### Windows Build on macOS/Linux

For Windows targets, use GitHub Actions or a Windows VM. Cross-compilation to Windows from Unix is complex and not recommended.

---

## Platform Support Matrix

| Platform | Target Triple | Status | Notes |
|----------|---------------|--------|-------|
| macOS Intel | x86_64-apple-darwin | ✅ Supported | Native build |
| macOS ARM | aarch64-apple-darwin | ✅ Supported | Native build |
| Linux x64 | x86_64-unknown-linux-gnu | ✅ Supported | Native build |
| Linux ARM64 | aarch64-unknown-linux-gnu | ✅ Supported | Use `cross` |
| Windows x64 | x86_64-pc-windows-msvc | ✅ Supported | Native build on Windows |
| Windows ARM64 | aarch64-pc-windows-msvc | ⚠️ Experimental | Not tested |
| FreeBSD | x86_64-unknown-freebsd | ❌ Not supported | Requires additional setup |

---

## Quick Reference

### Target Triples

```
macOS Intel:     x86_64-apple-darwin
macOS ARM:       aarch64-apple-darwin
Linux x64:       x86_64-unknown-linux-gnu
Linux ARM64:     aarch64-unknown-linux-gnu
Windows x64:     x86_64-pc-windows-msvc
Windows ARM64:   aarch64-pc-windows-msvc
```

### Build Commands

```bash
# Standard release build
cargo build --release

# Platform-specific build
cargo build --release --target <target-triple>

# Cross-compile Linux ARM64
cross build --release --target aarch64-unknown-linux-gnu

# Build with verbose output
cargo build --release --verbose

# Clean and rebuild
cargo clean && cargo build --release
```

### Binary Locations

```
target/<target-triple>/release/bitbucket-mcp        # macOS/Linux
target/<target-triple>/release/bitbucket-mcp.exe    # Windows
```
