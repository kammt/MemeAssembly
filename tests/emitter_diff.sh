#!/bin/sh
# Differential test of the Rust Emitter against GNU as (x86_64-linux-gnu-as, x86_64-w64-mingw32-as).
# Usage: tests/emitter_diff.sh [--quick] [--no-gen] [--out DIR] [--show N]
export PATH="$HOME/.cargo/bin:$PATH"
exec python3 "$(dirname "$0")/emitter_diff.py" "$@"
