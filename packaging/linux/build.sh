#!/usr/bin/env bash
# Compile natively on the oldest supported glibc, inside the pinned Ubuntu 22.04 image.
set -euo pipefail
case "${1:-}" in x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu) ;; *) exit 2 ;; esac
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y --no-install-recommends build-essential pkg-config ca-certificates git \
  libfontconfig1-dev libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev \
  libxi-dev libxrandr-dev libgl1-mesa-dev libvulkan-dev
cargo build --package app --bin tourne-la-page --release --locked --target "$1"
