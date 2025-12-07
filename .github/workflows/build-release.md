# Build Release Workflow

This document explains the `build-release.yml` GitHub Actions workflow that automates building and releasing FFNodes across multiple platforms and architectures.

## Overview

The workflow builds FFNodes for **three platforms** (Windows, Linux, macOS) and **two architectures** (x64, arm64) on each platform, resulting in **six different builds** per release. It then packages all artifacts and creates a GitHub release.

## Trigger Conditions

The workflow can be triggered in two ways:

1. **Manual Dispatch** - Run manually from the GitHub Actions tab
2. **Git Tags** - Automatically triggers when a version tag is pushed:
   - Standard releases: `v1.0.0`, `v2.1.3`
   - Pre-releases: `v1.0.0-beta`, `v2.0.0-rc.1`

```bash
# Example: Create and push a release tag
git tag v1.0.0
git push --tags
```

## Workflow Structure

The workflow consists of three main build jobs that run in parallel:

```
build-windows (x64 + arm64)
build-linux (x64 + arm64)
build-macos (x64 + arm64)
```

All artifacts are output to the `./dist/` directory in the repository root.

---

## Job 1: Build Windows

**Runs on:** `windows-latest`

**Matrix Strategy:** Builds for both `x64` and `arm64` architectures in parallel.

### Steps

1. **Checkout code** - Clones the repository

2. **Setup Rust** - Installs Rust toolchain with cross-compilation targets
   - x64: `x86_64-pc-windows-msvc`
   - arm64: `aarch64-pc-windows-msvc`

3. **Setup pnpm** - Installs pnpm v8 package manager

4. **Setup Node.js** - Installs Node.js 20 with pnpm caching

5. **Install frontend dependencies** - Runs `pnpm install` in `ffnodes-client/`

6. **Build packages** - Runs `cargo pkg --no-clean`
   - Sets `CARGO_BUILD_TARGET` environment variable
   - Package tool detects the target and passes it to Tauri and Cargo
   - Builds both client (Tauri app) and server
   - Creates installers: MSI and NSIS for Windows

7. **List built artifacts** - Lists all files created in `dist/` directory
   - Client ZIP: `ffnodes-client-{version}-windows-{arch}.zip`
   - MSI installer: `ffnodes_client_{version}_windows-{arch}_en-US.msi`
   - NSIS installer: `ffnodes_client_{version}_windows-{arch}_setup.exe`
   - Server ZIP: `ffnodes-server-{version}-windows-{arch}.zip`

   **Note:** Windows builds create both MSI and NSIS installers for maximum compatibility. All artifacts remain in the `./dist/` directory for collection.

---

## Job 2: Build Linux

**Runs on:** `ubuntu-latest`

**Matrix Strategy:** Builds for both `x64` and `arm64` architectures in parallel.

### Steps

1. **Checkout code** - Clones the repository

2. **Install system dependencies** - Installs Tauri dependencies
   - `libwebkit2gtk-4.1-dev` - WebView rendering
   - `libappindicator3-dev` - System tray support
   - `librsvg2-dev` - SVG rendering
   - `patchelf` - Binary patching utility

3. **Install ARM cross-compilation tools** (only for arm64 build)
   - Installs `gcc-aarch64-linux-gnu` and `g++-aarch64-linux-gnu`
   - Adds arm64 architecture to dpkg
   - Installs arm64 versions of Tauri dependencies

4. **Setup Rust** - Installs Rust toolchain with cross-compilation targets
   - x64: `x86_64-unknown-linux-gnu`
   - arm64: `aarch64-unknown-linux-gnu`

5. **Setup pnpm** - Installs pnpm v8 package manager

6. **Setup Node.js** - Installs Node.js 20 with pnpm caching

7. **Install frontend dependencies** - Runs `pnpm install` in `ffnodes-client/`

8. **Build packages** - Runs `cargo pkg --no-clean`
   - Sets `CARGO_BUILD_TARGET` environment variable
   - For arm64: Sets cross-compilation C/C++ compilers
   - Package tool handles target-specific build paths

9. **List built artifacts** - Lists all files created in `dist/` directory
   - Client ZIP: `ffnodes-client-{version}-linux-{arch}.zip`
   - AppImage: `ffnodes_client_{version}_linux-{arch}.AppImage`
   - DEB package: `ffnodes_client_{version}_linux-{arch}.deb`
   - Server ZIP: `ffnodes-server-{version}-linux-{arch}.zip`

   **Note:** Linux builds create both AppImage (portable) and DEB (Debian/Ubuntu) packages for maximum distribution compatibility. All artifacts remain in the `./dist/` directory for collection.

---

## Job 3: Build macOS

**Runs on:** `macos-latest`

**Matrix Strategy:** Builds for both `x64` (Intel) and `arm64` (Apple Silicon) in parallel.

### Steps

1. **Checkout code** - Clones the repository

2. **Setup Rust** - Installs Rust toolchain with cross-compilation targets
   - x64 (Intel): `x86_64-apple-darwin`
   - arm64 (Apple Silicon): `aarch64-apple-darwin`

3. **Setup pnpm** - Installs pnpm v8 package manager

4. **Setup Node.js** - Installs Node.js 20 with pnpm caching

5. **Install frontend dependencies** - Runs `pnpm install` in `ffnodes-client/`

6. **Build packages** - Runs `cargo pkg --no-clean`
   - Sets `CARGO_BUILD_TARGET` environment variable
   - Tauri automatically handles macOS universal binary creation if needed

7. **List built artifacts** - Lists all files created in `dist/` directory
   - Client ZIP: `ffnodes-client-{version}-macos-{arch}.zip`
   - DMG installer: `ffnodes_client_{version}_macos-{arch}.dmg`
   - APP bundle (tar.gz): `ffnodes_client_{version}_macos-{arch}.app.tar.gz`
   - Server ZIP: `ffnodes-server-{version}-macos-{arch}.zip`

   **Note:** macOS builds create both DMG (installer) and APP bundle (for manual installation) formats. All artifacts remain in the `./dist/` directory for collection.

---

## Collecting Build Artifacts

After all build jobs complete, artifacts from all platforms and architectures will be located in the `./dist/` directory at the repository root:

- Windows artifacts (MSI, NSIS, ZIP)
- Linux artifacts (AppImage, DEB, ZIP)
- macOS artifacts (DMG, APP bundle, ZIP)
- Server artifacts (ZIP for all platforms)

### Creating a GitHub Release

To create a release with these artifacts, you can:

1. **Manual Release**: Upload files from `./dist/` to a GitHub release manually
2. **gh CLI**: Use `gh release create v1.0.0 ./dist/*`
3. **Add release job**: Modify workflow to add a release creation step that uploads `./dist/*` files

---

## Cross-Compilation Details

### How It Works

The workflow leverages the `CARGO_BUILD_TARGET` environment variable, which the `tools/package` utility detects and uses to:

1. Pass `--target` flag to Tauri CLI: `tauri build --target {arch}`
2. Pass `--target` flag to Cargo: `cargo build --release --target {arch}`
3. Adjust output paths: `target/{arch}/release/` instead of `target/release/`
4. Generate platform/arch-specific filenames

### Platform-Specific Notes

**Windows:**
- Native x64 builds run directly on Windows runners
- arm64 cross-compilation uses MSVC toolchain
- Generates both MSI and NSIS installers

**Linux:**
- Native x64 builds run directly on Ubuntu runners
- arm64 cross-compilation requires GCC cross-compiler
- Must install arm64 system libraries for linking

**macOS:**
- GitHub-hosted macOS runners are Apple Silicon (arm64)
- Can cross-compile to x64 (Intel) without additional tools
- Apple provides native cross-compilation support

---

## Artifact Structure

Each build produces the following files:

### Client Artifacts

**All Platforms:**
- **ZIP**: Portable executable (no installation required)
  - `ffnodes-client-{version}-{platform}-{arch}.zip`

**Windows Only:**
- **MSI Installer**: Windows Installer package
  - `ffnodes_client_{version}_windows-{arch}_en-US.msi`
- **NSIS Installer**: Nullsoft scriptable install system
  - `ffnodes_client_{version}_windows-{arch}_setup.exe`

**Linux Only:**
- **AppImage**: Portable application format
  - `ffnodes_client_{version}_linux-{arch}.AppImage`
- **DEB Package**: Debian/Ubuntu package
  - `ffnodes_client_{version}_linux-{arch}.deb`

**macOS Only:**
- **DMG**: Disk image installer
  - `ffnodes_client_{version}_macos-{arch}.dmg`
- **APP Bundle**: Application bundle (tar.gz)
  - `ffnodes_client_{version}_macos-{arch}.app.tar.gz`

### Server Artifacts
- **ZIP**: Portable server executable
  - `ffnodes-server-{version}-{platform}-{arch}.zip`

### Total Artifacts per Release
- **6** client ZIPs (3 platforms × 2 architectures)
- **4** Windows installers (2 MSI + 2 NSIS for x64/arm64)
- **4** Linux packages (2 AppImage + 2 DEB for x64/arm64)
- **4** macOS packages (2 DMG + 2 APP for x64/arm64)
- **6** server ZIPs (3 platforms × 2 architectures)
- **24 files total**

---

## Version Detection

The package tool automatically extracts version numbers from:
- **Client**: `ffnodes-client/src-tauri/Cargo.toml`
- **Server**: `ffnodes-server/Cargo.toml`

Both must use semantic versioning (e.g., `0.1.41-beta`).

---

## Dependencies

### Required on Build Runners
- Rust toolchain with cross-compilation targets
- pnpm ≥ 8.13.1
- Node.js (latest LTS recommended)
- Platform-specific build tools (installed by workflow)

### Required in Repository
- `tools/package` - Custom build/package utility
- `.cargo/config.toml` - Defines `cargo pkg` alias
- Tauri configuration in `ffnodes-client/src-tauri/`

---

## Usage Examples

### Create a Standard Release
```bash
git tag v1.0.0
git push --tags
```

### Create a Pre-release
```bash
git tag v1.0.0-beta
git push --tags
```

### Manual Trigger
1. Go to GitHub Actions tab
2. Select "build-release.yml"
3. Click "Run workflow"
4. Choose branch and click "Run"

### Local Testing with `act`

To test the workflow locally and have artifacts persist to your host machine:

```bash
# Create dist directory (if it doesn't exist)
mkdir -p dist

# Run with bind mount to ensure artifacts persist
gh act -W .github/workflows/build-release.yml workflow_dispatch --matrix arch:x64 -j build-linux --bind

# Check artifacts in ./dist/
ls -lah dist/
```

**Note:** When using `act`, the workflow runs in a Docker container. The `--bind` flag ensures the repository directory (including `dist/`) is properly mounted from your host machine, allowing artifacts to persist after the container exits.

**Alternative:** Use PowerShell to copy artifacts from container:
```powershell
# After build completes, find the container and copy artifacts
docker ps -a  # Find the container ID
docker cp <container-id>:/workspace/dist ./dist
```

---

## Troubleshooting

### Build Failures

**Symptom:** Build fails with missing dependencies

**Solution:** Ensure all required system dependencies are installed in workflow

**Symptom:** Cross-compilation fails on Linux arm64

**Solution:** Verify arm64 system libraries are installed and cross-compiler env vars are set

### Artifact Upload Failures

**Symptom:** `if-no-files-found: error` triggers

**Solution:** Check that `cargo pkg` successfully creates files in `dist/`

**Symptom:** Wrong architecture binaries uploaded

**Solution:** Verify `CARGO_BUILD_TARGET` is correctly set and respected by build tool

### Release Creation Failures

**Symptom:** Release not created or missing files

**Solution:** Ensure `contents: write` permission is granted and all build jobs succeeded

---

## Maintenance

### Updating Platform Support

To add a new platform (e.g., FreeBSD):

1. Add a new `build-freebsd` job with appropriate matrix
2. Install platform-specific dependencies
3. Add the job to `needs:` array in `create-release` job
4. Update artifact download steps

### Updating Architecture Support

To add a new architecture (e.g., riscv64):

1. Add to matrix: `arch: [x64, arm64, riscv64]`
2. Add target mapping in matrix include
3. Update cross-compilation setup if needed
4. Update `get_platform_arch()` in `tools/package/src/main.rs`

---

## Security Considerations

- Workflow uses `GITHUB_TOKEN` with `contents: write` permission
- No secrets or credentials are required
- Build artifacts are public (uploaded to public GitHub release)
- Cross-compilation may execute untrusted code (review dependencies carefully)

---

## Performance

**Typical Build Times:**
- Windows: ~10-15 minutes per architecture
- Linux: ~10-15 minutes per architecture
- macOS: ~15-20 minutes per architecture

**Total Workflow Duration:** ~20-25 minutes (builds run in parallel)

**Caching:**
- pnpm dependencies cached between runs
- Cargo registry/target not cached (would be huge)

---

## Related Documentation

- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [Tauri Build Documentation](https://tauri.app/v1/guides/building/)
- [GitHub Actions Matrix Strategy](https://docs.github.com/en/actions/using-jobs/using-a-matrix-for-your-jobs)
- [softprops/action-gh-release](https://github.com/softprops/action-gh-release)
