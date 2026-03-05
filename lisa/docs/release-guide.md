# Release Guide

How to build and publish Lisa pre-built binaries via GitHub Releases.

## Why GitHub Releases?

- **DO NOT** commit binaries to the git repo (bloats history permanently)
- GitHub Releases stores binaries separately from git history
- Users download only what they need
- Version-tagged and easy to manage

## Build

### ARM64 (webOS TV target)
```bash
# Cross-compile (requires aarch64 toolchain)
cargo build --release --target aarch64-unknown-linux-musl

# Binary location
ls target/aarch64-unknown-linux-musl/release/zeroclaw
```

### x86_64 (Linux server)
```bash
cargo build --release --target x86_64-unknown-linux-musl

ls target/x86_64-unknown-linux-musl/release/zeroclaw
```

### macOS (Apple Silicon)
```bash
cargo build --release

ls target/release/zeroclaw
```

## Publish a Release

### Using gh CLI
```bash
# Create release with binaries
gh release create v0.1.8 \
  target/aarch64-unknown-linux-musl/release/zeroclaw#zeroclaw-arm64-linux \
  target/x86_64-unknown-linux-musl/release/zeroclaw#zeroclaw-x86_64-linux \
  --repo rordd/lisa \
  --title "Lisa v0.1.8" \
  --notes "Release notes here"
```

### Including gog CLI
```bash
gh release create v0.1.8 \
  target/aarch64-unknown-linux-musl/release/zeroclaw#zeroclaw-arm64-linux \
  target/x86_64-unknown-linux-musl/release/zeroclaw#zeroclaw-x86_64-linux \
  /path/to/gog-arm64#gog-arm64-linux \
  /path/to/gog-x86_64#gog-x86_64-linux \
  --repo rordd/lisa \
  --title "Lisa v0.1.8" \
  --notes "Release notes here"
```

### Format
```
gh release create <tag> <file>#<display-name> [<file>#<display-name> ...]
```

## Download a Release

### From CLI
```bash
# List releases
gh release list --repo rordd/lisa

# Download specific release
gh release download v0.1.8 --repo rordd/lisa --dir ./bin/

# Download specific asset
gh release download v0.1.8 --repo rordd/lisa --pattern "zeroclaw-arm64-linux"
```

### From browser
Go to: https://github.com/rordd/lisa/releases

## Deploy Script Integration

Deploy scripts should download from releases instead of bundling binaries:

```bash
# In deploy-target.sh
VERSION="${LISA_VERSION:-latest}"
if [ "$VERSION" = "latest" ]; then
    gh release download --repo rordd/lisa --pattern "zeroclaw-arm64-linux" --dir /tmp/
else
    gh release download "$VERSION" --repo rordd/lisa --pattern "zeroclaw-arm64-linux" --dir /tmp/
fi
chmod +x /tmp/zeroclaw-arm64-linux
```

## Naming Convention

| Platform | Binary name |
|----------|-------------|
| ARM64 Linux (webOS TV) | `zeroclaw-arm64-linux` |
| x86_64 Linux (server) | `zeroclaw-x86_64-linux` |
| macOS ARM64 | `zeroclaw-macos-arm64` |
| gog ARM64 Linux | `gog-arm64-linux` |
| gog x86_64 Linux | `gog-x86_64-linux` |

## Version Tags

Follow semver: `v0.1.8`, `v0.2.0`, etc.

```bash
git tag v0.1.8
git push origin v0.1.8
gh release create v0.1.8 ...
```
