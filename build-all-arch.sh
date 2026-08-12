#!/usr/bin/env bash
# Multi-Architecture & Multi-OS Master Release Builder for Acer Monitor CLI (amctl)
set -e

mkdir -p dist

echo "🚀 Building [1/4] AMD64 Linux Native Binary & AppImage..."
cargo build --release --target x86_64-unknown-linux-gnu
cp target/x86_64-unknown-linux-gnu/release/acer_monitor_cli dist/amctl-amd64-linux
chmod +x build-appimage.sh && ./build-appimage.sh
cp amctl-x86_64.AppImage dist/amctl-amd64.AppImage

echo "🚀 Building [2/4] Windows 11 / 10 AMD64 Native EXE..."
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/acer_monitor_cli.exe dist/acer_monitor_cli-amd64-win11.exe

if command -v aarch64-linux-gnu-gcc &> /dev/null; then
    echo "🚀 Building [3/4] ARM64 Linux (Raspberry Pi 4/5 & ARM64 Servers)..."
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    cargo build --release --target aarch64-unknown-linux-gnu
    cp target/aarch64-unknown-linux-gnu/release/acer_monitor_cli dist/amctl-arm64-linux
fi

if command -v riscv64-linux-gnu-gcc &> /dev/null; then
    echo "🚀 Building [4/4] RISC-V 64 Linux..."
    CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_LINKER=riscv64-linux-gnu-gcc \
    cargo build --release --target riscv64gc-unknown-linux-gnu
    cp target/riscv64gc-unknown-linux-gnu/release/acer_monitor_cli dist/amctl-riscv64-linux
fi

echo "✅ All multi-architecture binaries packaged in dist/:"
ls -lh dist/
