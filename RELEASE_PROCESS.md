# AOF Release Process

This document outlines how to build, test, and publish AOF binaries to make them available via the installation script at https://aof.sh/install.sh

## Overview

The release process is fully automated via GitHub Actions. When you create a git tag, GitHub Actions will:

1. Build binaries for multiple platforms
2. Calculate checksums
3. Create a GitHub release with assets
4. Deploy the install script

## Prerequisites

- Git with push access to https://github.com/agenticopsorg/aof
- Rust toolchain installed locally (for testing builds)
- GitHub Actions enabled on the repository

## Release Process

### Step 1: Update Version Numbers

Update the version in `aof/Cargo.toml`:

```toml
[workspace.package]
version = "0.2.0"  # Change this
```

### Step 2: Build and Test Locally (Optional)

Test the build locally before releasing:

```bash
# Build for current platform
cd aof
cargo build --release --package aofctl

# Test the binary
./target/release/aofctl --version
./target/release/aofctl api-resources
```

### Step 3: Create a Release Tag

```bash
# Create and push a version tag
git tag -a v0.2.0 -m "Release v0.2.0: Description of changes"
git push origin v0.2.0
```

This triggers the GitHub Actions workflow automatically.

### Step 4: Monitor the Build

1. Go to: https://github.com/agenticopsorg/aof/actions
2. Watch the "Build and Release Binaries" workflow
3. Wait for all platforms to build successfully
4. The workflow will create a GitHub Release with all binaries

### Step 5: Verify the Release

Once complete, verify at: https://github.com/agenticopsorg/aof/releases

You should see:
- Binaries for each platform (linux-x86_64, macos-x86_64, etc.)
- SHA256 checksums for each binary
- Release notes

### Step 6: Deploy install.sh

The install script needs to be accessible at https://aof.sh/install.sh

#### Option A: GitHub Pages (Simple)

1. Create a `gh-pages` branch
2. Push the install script there
3. Configure GitHub Pages to serve from `/scripts/install.sh`

#### Option B: Dedicated Web Server (Recommended)

1. Copy `scripts/install.sh` to your web server
2. Make it accessible at `https://aof.sh/install.sh`
3. Ensure proper HTTPS and CORS headers

#### Option C: GitHub Raw Content

```
https://raw.githubusercontent.com/agenticopsorg/aof/main/scripts/install.sh
```

## Testing the Installation

After releasing, test the installation script:

```bash
# Test with latest version
curl -sSL https://aof.sh/install.sh | bash

# Test with specific version
curl -sSL https://aof.sh/install.sh | bash -s -- --version v0.2.0

# Test with custom install directory
curl -sSL https://aof.sh/install.sh | bash -s -- --install-dir /usr/local/bin

# Verbose output for debugging
curl -sSL https://aof.sh/install.sh | bash -s -- --verbose
```

## Supported Platforms

The release workflow builds for:

- **Linux**
  - x86_64 (Intel/AMD 64-bit)
  - aarch64 (ARM 64-bit)

- **macOS**
  - x86_64 (Intel)
  - aarch64 (Apple Silicon M1/M2/M3)

- **Windows**
  - x86_64 (Intel/AMD 64-bit)

## Installation Script Features

The `install.sh` script:

- ✅ Auto-detects your OS and CPU architecture
- ✅ Downloads the correct binary from GitHub releases
- ✅ Verifies checksums (if sha256sum available)
- ✅ Installs to ~/.local/bin by default
- ✅ Makes binary executable
- ✅ Warns if binary not in PATH
- ✅ Verifies successful installation
- ✅ Provides helpful next steps

## Troubleshooting

### Build fails for a platform

1. Check the Actions log: https://github.com/agenticopsorg/aof/actions
2. Common issues:
   - Cross-compilation tools missing (automatic via `cross`)
   - Rust target not installed (automatic)
   - Dependency version conflicts (check Cargo.toml)

### Installation script fails

1. Check network connectivity
2. Verify GitHub releases are publicly accessible
3. Run with `--verbose` flag: `bash -s -- --verbose`
4. Check logs in temp directory

### Binary doesn't work after installation

1. Verify architecture: `uname -m`
2. Check file permissions: `ls -l ~/.local/bin/aofctl`
3. Verify binary works: `~/.local/bin/aofctl --version`
4. Check PATH: `echo $PATH | grep .local/bin`

## Manual Release (If Automation Fails)

If GitHub Actions fails, you can manually:

1. Build locally:
   ```bash
   cargo build --release --package aofctl
   ```

2. Create checksums:
   ```bash
   sha256sum target/release/aofctl > aofctl.sha256
   ```

3. Upload to GitHub Releases:
   ```bash
   gh release create v0.2.0 target/release/aofctl aofctl.sha256 \
     --title "AOF v0.2.0" \
     --notes "Release notes here"
   ```

4. Update install script location

## Release Checklist

Before each release:

- [ ] Update version in `aof/Cargo.toml`
- [ ] Update CHANGELOG.md (if exists)
- [ ] Commit changes
- [ ] Run tests: `cargo test --package aofctl`
- [ ] Build locally: `cargo build --release --package aofctl`
- [ ] Test locally: `./target/release/aofctl --version`
- [ ] Create git tag: `git tag -a v0.2.0 -m "Release message"`
- [ ] Push tag: `git push origin v0.2.0`
- [ ] Monitor GitHub Actions build
- [ ] Verify GitHub Release created
- [ ] Test installation script
- [ ] Update documentation links if needed

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR.MINOR.PATCH** (e.g., 1.2.3)
- MAJOR: Breaking changes
- MINOR: New features
- PATCH: Bug fixes

Examples:
- `v0.1.0` - Initial release
- `v0.1.1` - Bug fix
- `v0.2.0` - New features
- `v1.0.0` - Stable release

## Continuous Deployment

The installation script always downloads from GitHub Releases. To automatically use the latest version:

```bash
curl -sSL https://aof.sh/install.sh | bash
```

To pin to a specific version:

```bash
curl -sSL https://aof.sh/install.sh | bash -s -- --version v0.1.0
```

## Next Steps

1. Configure `aof.sh` to serve `scripts/install.sh` at `/install.sh`
2. Create first release tag: `git tag -a v0.1.0 -m "Initial release"`
3. Push tag to trigger build: `git push origin v0.1.0`
4. Monitor build at: https://github.com/agenticopsorg/aof/actions
5. Verify release: https://github.com/agenticopsorg/aof/releases
6. Test installation

## Support

For issues with the release process:
1. Check GitHub Actions logs
2. Review this document
3. Open an issue: https://github.com/agenticopsorg/aof/issues
