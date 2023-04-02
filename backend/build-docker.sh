#!/bin/bash

echo "--- Rust version info ---"
rustup --version
rustc --version
cargo --version

mkdir -p out

echo "--- Building ryzenadj lib ---"
git clone https://github.com/FlyGoat/RyzenAdj ryzenadj
cd ryzenadj
git checkout -q 160502771054d31d2f4c2fa46ad42c96336f3a74
mkdir build && cd build
cmake -DCMAKE_BUILD_TYPE=Release ..
make
mv libryzenadj.so ../../out/libryzenadj.so
cd ../..

echo "--- Building plugin backend ---"
cargo build --profile docker
mkdir -p out

mv target/docker/powertools out/backend

echo " --- Cleaning up ---"
# remove root-owned target folder
cargo clean
# remove newly-cloned git repo and artifacts
rm -rf ./ryzenadj
