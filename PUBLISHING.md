# Publishing to crates.io

This guide explains how to publish `kadabra-runes` to crates.io so users can install it with `cargo install kadabra-runes`.

## Prerequisites

### 1. Create a crates.io Account

1. Go to https://crates.io/
2. Click "Log in with GitHub" in the top right
3. Authorize crates.io to access your GitHub account

### 2. Get Your API Token

1. After logging in, go to https://crates.io/me
2. Scroll to "API Tokens"
3. Click "New Token"
4. Give it a name (e.g., "kadabra-runes-publish")
5. Click "Generate" and copy the token
6. **Save this token securely** - you'll only see it once!

### 3. Configure Cargo with Your Token

```bash
cargo login YOUR_API_TOKEN_HERE
```

This stores your token in `~/.cargo/credentials` for future use.

## Pre-Publication Checklist

✅ **Already Done:**
- [x] Package metadata complete in Cargo.toml
- [x] LICENSE file created (MIT)
- [x] README.md exists
- [x] All tests pass
- [x] No compiler warnings

✅ **Verify Before Publishing:**

```bash
# 1. Ensure you're on the latest main branch
git checkout main
git pull origin main

# 2. Run all tests
cargo test

# 3. Build in release mode
cargo build --release

# 4. Check package contents (dry run)
cargo package --list

# 5. Perform a dry-run publish (doesn't actually publish)
cargo publish --dry-run

# 6. Check the package builds from the packaged source
cargo package --allow-dirty
tar -xzf target/package/kadabra-runes-0.1.0.crate -C /tmp
cd /tmp/kadabra-runes-0.1.0
cargo build
cd -
```

## Publishing Steps

### First-Time Publish

```bash
# 1. Make sure everything is committed
git status

# 2. Publish to crates.io
cargo publish

# 3. Wait for indexing (takes a few minutes)
# Check https://crates.io/crates/kadabra-runes

# 4. Tag the release
git tag v0.1.0
git push origin v0.1.0

# 5. Create GitHub Release (CI will do this automatically)
```

### Publishing Updates (Automated with cargo-release)

#### Prerequisites (One-time Setup)

1. **Install cargo-release:**
   ```bash
   cargo install cargo-release
   ```

2. **Configure cargo with your crates.io token:**
   ```bash
   cargo login YOUR_API_TOKEN_HERE
   ```

3. **Verify CARGO_TOKEN GitHub secret is set** for binary builds

#### Release Process

```bash
# 1. Ensure you're on main and up to date
git checkout main
git pull origin main

# 2. Update the [Unreleased] section in CHANGELOG.md with your changes
vim CHANGELOG.md

# 3. Commit the CHANGELOG updates
git add CHANGELOG.md
git commit -m "Update CHANGELOG for upcoming release"
git push origin main

# 4. Run cargo-release (dry-run first is recommended)
cargo release patch --dry-run  # Review what will happen
cargo release patch --execute  # Execute the release

# For different version bumps:
# cargo release minor --execute  # 0.1.3 → 0.2.0
# cargo release major --execute  # 0.1.3 → 1.0.0
```

**What cargo-release does:**
1. Bumps version in Cargo.toml
2. Moves [Unreleased] section to versioned section in CHANGELOG.md
3. Creates a git commit with all changes
4. Publishes to crates.io
5. Creates and pushes a git tag
6. Pushes the commit to origin
7. GitHub Actions automatically builds binaries and creates GitHub Release

#### Version Bump Semantics

- **patch** (0.1.3 → 0.1.4): Bug fixes, minor improvements
- **minor** (0.1.3 → 0.2.0): New features, backward-compatible changes
- **major** (0.1.3 → 1.0.0): Breaking changes

#### Troubleshooting

**"error: repository is dirty"**
```bash
git status
git add .
git commit -m "Prepare for release"
```

**"error: not on allowed branch"**
```bash
git checkout main
```

**"error: failed to publish"**
```bash
cargo login --check
cargo login YOUR_TOKEN  # Re-login if needed
```

**Undo a failed release:**
```bash
# Delete the local tag
git tag -d v0.1.4

# Delete the remote tag (if it was pushed)
git push origin :refs/tags/v0.1.4

# Reset to the commit before the release
git reset --hard origin/main

# If already published to crates.io, yank it
cargo yank --vers 0.1.4
```

## Troubleshooting

### "error: package has uncommitted changes"

```bash
# Either commit your changes:
git add .
git commit -m "Prepare for release"

# Or use --allow-dirty (not recommended):
cargo publish --allow-dirty
```

### "error: failed to verify package tarball"

This usually means the build fails from the packaged source. Check that all files needed for building are included:

```bash
# Check what's being packaged
cargo package --list

# Ensure no required files are excluded
# Check .gitignore and Cargo.toml's [package] exclude/include
```

### "error: crate name already exists"

The name `kadabra-runes` might be taken. You'll need to either:
1. Choose a different name in Cargo.toml
2. Contact the current owner if it's squatted
3. Use a different prefix like `kadabra-runes-mcp`

### "error: authentication token is invalid"

Your token expired or is incorrect:

```bash
# Generate a new token at https://crates.io/me
cargo login YOUR_NEW_TOKEN
```

## After Publishing

### Verify Installation

```bash
# Wait 1-2 minutes for crates.io to index, then test:
cargo install kadabra-runes

# Verify it works
kadabra-runes --version
kadabra-runes --help
```

### Update Documentation

Update README.md to include crates.io installation:

```markdown
## Installation

### From crates.io (Recommended)

\`\`\`bash
cargo install kadabra-runes
\`\`\`

### From source

\`\`\`bash
git clone https://github.com/kadabra-ai/kadabra-runes
cd kadabra-runes
cargo install --path .
\`\`\`
```

### Badge for README

Add this badge to README.md:

```markdown
[![Crates.io](https://img.shields.io/crates/v/kadabra-runes.svg)](https://crates.io/crates/kadabra-runes)
```

## Automatic Publishing via GitHub Actions

The release workflow (`.github/workflows/release.yml`) includes automatic publishing to crates.io.

**To enable:**

1. Go to GitHub repository settings
2. Navigate to Secrets and variables → Actions
3. Add new secret: `CARGO_TOKEN`
4. Paste your crates.io API token
5. Now pushing a tag will automatically publish to crates.io!

## Useful Commands

```bash
# Check current crate info on crates.io
cargo search kadabra-runes

# View download stats
# Visit: https://crates.io/crates/kadabra-runes

# Yank a version (makes it unavailable for new installs, but doesn't break existing users)
cargo yank --vers 0.1.0

# Unyank a version
cargo yank --vers 0.1.0 --undo
```

## Important Notes

- ⚠️ **You cannot delete a version once published** - only yank it
- ⚠️ **You cannot republish the same version** - must bump version
- ✅ **Yanking doesn't break existing users** - they can still use that version
- ✅ **Publishing is permanent** - make sure you're ready!
- ✅ **Reserve the name early** - publish v0.1.0 quickly if needed

## Resources

- [Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Semantic Versioning](https://semver.org/)
- [crates.io Policies](https://crates.io/policies)
