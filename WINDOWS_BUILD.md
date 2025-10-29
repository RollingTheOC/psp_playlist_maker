# Building for Windows from Linux

## Method 1: Using MinGW Cross-Compiler (Recommended)

### Install Dependencies
```bash
# Install MinGW cross-compiler
sudo apt-get update
sudo apt-get install mingw-w64

# Add Windows target to Rust
rustup target add x86_64-pc-windows-gnu
```

### Build
```bash
cargo build --release --target x86_64-pc-windows-gnu
```

The Windows binary will be at: `target/x86_64-pc-windows-gnu/release/psp_playlist_maker.exe`

## Method 2: Native Windows Build

### On Windows:

1. Install Rust from: https://rustup.rs/
2. Install Visual Studio Build Tools or Visual Studio (C++ development tools)
3. Clone the repository
4. Build:
```powershell
cargo build --release
```

The binary will be at: `target\release\psp_playlist_maker.exe`

## Method 3: GitHub Actions (Automated)

Use the provided `.github/workflows/release.yml` to automatically build for both platforms when you create a GitHub release.

## Dependencies for Windows Build

The Windows build will need these DLLs (usually included with Windows):
- OpenSSL DLLs (if not statically linked)
- Visual C++ Runtime

Consider using `cargo-bundle` or `cargo-wix` for creating Windows installers.

## Static Linking (Recommended for Distribution)

To create a standalone Windows executable without DLL dependencies, add to `Cargo.toml`:

```toml
[target.x86_64-pc-windows-gnu]
rustflags = ["-C", "link-arg=-static"]
```

Or build with:
```bash
RUSTFLAGS="-C link-arg=-static" cargo build --release --target x86_64-pc-windows-gnu
```
