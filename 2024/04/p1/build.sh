#!/usr/bin/env bash
set -euxo pipefail

rustup run nightly rustc \
    --target x86_64-unknown-none \
    -Crelocation-model=static \
    \
    -Copt-level=3 \
    -Ctarget-cpu=native \
    -Cpanic=abort \
    -Cstrip=symbols \
    -Coverflow_checks=n \
    -Clto \
    \
    $* -o main main.rs
