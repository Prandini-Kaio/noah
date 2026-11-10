#!/bin/sh
set -eu
# Empacotamento para desenvolvedores; o usuário final abre o executável NOAH.
repo=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$repo"
cargo build --release --locked -p noah-desktop
binary_root=${CARGO_TARGET_DIR:-"$repo/target"}
output=${1:-"$repo/dist/noah-linux-x86_64"}
mkdir -p "$output"
cp "$binary_root/release/noah-desktop" "$output/NOAH"
chmod 755 "$output/NOAH"
cp LICENSE "$output/LICENSE"
cp docs/quality/THIRD-PARTY-E3.md "$output/DEPENDENCIAS.md"
cp -R docs/quality/third-party-licenses "$output/third-party-licenses"
cp docs/desktop/LEIA-ME-LINUX.txt "$output/LEIA-ME.txt"
printf 'Pacote preparado em %s\n' "$output"
