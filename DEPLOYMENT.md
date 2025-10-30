# Deployment Guide

This guide explains how to deploy and publish Inspector HTTP.

## Prerequisites

1. **GitHub Repository**: Repository must be set up with proper metadata in `Cargo.toml`
2. **Crates.io Account**: Required for publishing to crates.io
3. **GitHub Secrets**: Configure the following secrets in your repository:
   - `CARGO_REGISTRY_TOKEN`: Your crates.io API token (get it from https://crates.io/me)

## Publishing a Release

### 1. Update Version

Update the version in `Cargo.toml`:

```toml
[package]
version = "0.2.0"  # Update this
```

### 2. Update CHANGELOG.md

Add a new section with your changes:

```markdown
## [0.2.0] - 2025-11-01

### Added
- New feature X
- New feature Y

### Fixed
- Bug Z
```

### 3. Commit Changes

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"
git push origin master
```

### 4. Create and Push Git Tag

```bash
git tag v0.2.0
git push origin v0.2.0
```

### 5. Automated Release Process

When you push a tag matching `v*.*.*`, the GitHub Actions workflow will automatically:

1. **Build Binaries**: Creates optimized binaries for:
   - Linux (x86_64)
   - macOS (x86_64 and ARM64)
   - Windows (x86_64)

2. **Create GitHub Release**:
   - Creates a new release on GitHub
   - Attaches compiled binaries
   - Extracts changelog from CHANGELOG.md

3. **Publish to crates.io**:
   - Publishes the crate to crates.io
   - Makes it available via `cargo install inspector-http`

## Manual Publishing

If you need to publish manually to crates.io:

```bash
# Verify the package first
cargo package --allow-dirty

# Login to crates.io
cargo login <your-token>

# Publish
cargo publish
```

## First-Time Setup

### 1. Update Repository URL

In `Cargo.toml`, update:

```toml
repository = "https://github.com/YOUR_USERNAME/inspector-http"
authors = ["Your Name <your.email@example.com>"]
```

### 2. Create Crates.io Token

1. Go to https://crates.io/settings/tokens
2. Create a new token with publish permissions
3. Add it to GitHub Secrets as `CARGO_REGISTRY_TOKEN`

### 3. Verify GitHub Actions

Ensure your repository has Actions enabled:
- Go to Settings → Actions → General
- Allow all actions and reusable workflows

## Troubleshooting

### Build Failures

If builds fail for a specific platform:
- Check the Actions logs
- Linux builds may need additional dependencies
- macOS ARM builds require specific toolchain

### Publish Failures

If crates.io publish fails:
- Verify `CARGO_REGISTRY_TOKEN` is set correctly
- Check that the version doesn't already exist
- Ensure all metadata fields are valid

### Binary Size

The release binaries are optimized. If size is a concern:
- Strip symbols (already done in workflow)
- Use UPX compression (optional)
- Enable LTO in Cargo.toml

## Release Checklist

- [ ] Update version in Cargo.toml
- [ ] Update CHANGELOG.md
- [ ] Test build locally: `cargo build --release`
- [ ] Commit and push changes
- [ ] Create and push git tag
- [ ] Verify GitHub Actions workflow completes
- [ ] Test published crate: `cargo install inspector-http`
- [ ] Verify GitHub Release artifacts are attached
