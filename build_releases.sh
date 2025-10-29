#!/bin/bash
# Build script for creating release binaries

set -e

echo "🔨 Building PSP Playlist Maker releases..."
echo ""

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean

# Build for Linux (native)
echo ""
echo "📦 Building for Linux x86_64..."
cargo build --release
LINUX_BINARY="target/release/psp_playlist_maker"

# Create release directory
mkdir -p releases
rm -rf releases/*

# Package Linux release
echo ""
echo "📦 Packaging Linux release..."
mkdir -p releases/psp_playlist_maker-linux-x64
cp "$LINUX_BINARY" releases/psp_playlist_maker-linux-x64/
cp README.md releases/psp_playlist_maker-linux-x64/
cp LICENSE releases/psp_playlist_maker-linux-x64/
cp QUICKSTART.md releases/psp_playlist_maker-linux-x64/
cd releases
tar -czf psp_playlist_maker-v0.1.0-linux-x64.tar.gz psp_playlist_maker-linux-x64/
cd ..
echo "✅ Linux release created: releases/psp_playlist_maker-v0.1.0-linux-x64.tar.gz"

# Check if cross-compilation is available for Windows
if command -v cargo &> /dev/null; then
    echo ""
    echo "📦 Building for Windows x86_64..."
    
    # Check if Windows target is installed
    if rustup target list | grep -q "x86_64-pc-windows-gnu (installed)"; then
        echo "Windows target already installed"
    else
        echo "Installing Windows target..."
        rustup target add x86_64-pc-windows-gnu
    fi
    
    # Try to build for Windows
    if cargo build --release --target x86_64-pc-windows-gnu 2>/dev/null; then
        WINDOWS_BINARY="target/x86_64-pc-windows-gnu/release/psp_playlist_maker.exe"
        
        echo ""
        echo "📦 Packaging Windows release..."
        mkdir -p releases/psp_playlist_maker-windows-x64
        cp "$WINDOWS_BINARY" releases/psp_playlist_maker-windows-x64/
        cp README.md releases/psp_playlist_maker-windows-x64/
        cp LICENSE releases/psp_playlist_maker-windows-x64/
        cp QUICKSTART.md releases/psp_playlist_maker-windows-x64/
        cd releases
        zip -r psp_playlist_maker-v0.1.0-windows-x64.zip psp_playlist_maker-windows-x64/
        cd ..
        echo "✅ Windows release created: releases/psp_playlist_maker-v0.1.0-windows-x64.zip"
    else
        echo "⚠️  Windows cross-compilation failed. You may need mingw-w64 installed."
        echo "   Install with: sudo apt-get install mingw-w64"
        echo "   Then run: rustup target add x86_64-pc-windows-gnu"
    fi
fi

# Summary
echo ""
echo "🎉 Release build complete!"
echo ""
echo "Created releases:"
ls -lh releases/*.tar.gz releases/*.zip 2>/dev/null || ls -lh releases/*.tar.gz 2>/dev/null
echo ""
echo "Binary sizes:"
du -h "$LINUX_BINARY" 2>/dev/null || true
du -h "target/x86_64-pc-windows-gnu/release/psp_playlist_maker.exe" 2>/dev/null || true
