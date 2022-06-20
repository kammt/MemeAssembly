#!/usr/bin/env bash
set -euo pipefail

cd /opt
git clone https://github.com/emscripten-core/emsdk.git

cd emsdk

./emsdk install latest
./emsdk activate latest

export PATH="/opt/emsdk/upstream/emscripten/:$PATH"
