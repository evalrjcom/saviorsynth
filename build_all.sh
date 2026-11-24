#!/usr/bin/env bash
# Builds the Linux and Windows (x86_64, static, mingw) binaries into ./dist
# Needs: rust (rustup), cmake, a C++ compiler, ALSA + X11 dev packages, gcc-mingw-w64-x86-64 / g++-mingw-w64-x86-64
#   Debian/Ubuntu: sudo apt install build-essential cmake pkg-config libasound2-dev libx11-dev libxft-dev \
#                  libxinerama-dev libxcursor-dev libxfixes-dev libxrender-dev libpango1.0-dev libgl1-mesa-dev \
#                  libglu1-mesa-dev gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64
# The mingw toolchain must use POSIX threads:
#   sudo update-alternatives --set x86_64-w64-mingw32-gcc /usr/bin/x86_64-w64-mingw32-gcc-posix
#   sudo update-alternatives --set x86_64-w64-mingw32-g++ /usr/bin/x86_64-w64-mingw32-g++-posix
set -euo pipefail
cd "$(dirname "$0")"
NAME=$(sed -n 's/^name *= *"\(.*\)"/\1/p' Cargo.toml | head -1)
VER=$(sed -n 's/^version *= *"\(.*\)"/\1/p' Cargo.toml | head -1)
mkdir -p dist
WIN=x86_64-pc-windows-gnu

echo "== Linux =="
cargo build --release
cp "target/release/$NAME" "dist/${NAME}_linux_x86_64"

echo "== Windows =="
if command -v rustup >/dev/null 2>&1 && ! rustup target list --installed | grep -qx "$WIN"; then
  rustup target add "$WIN" || echo "rustup target add failed (offline?) - trying build-std fallback"
fi
if command -v rustup >/dev/null 2>&1 && rustup target list --installed | grep -qx "$WIN"; then
  cargo build --release --target "$WIN"
else
  # fallback: compile std from source (needs the rust-src component or a checkout linked into the sysroot)
  RUSTC_BOOTSTRAP=1 cargo build --release --target "$WIN" -Zbuild-std=std,panic_abort
fi
cp "target/$WIN/release/$NAME.exe" "dist/${NAME}_windows_x64.exe"

ls -la dist
echo "built $NAME $VER"
