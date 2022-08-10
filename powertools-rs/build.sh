#!/bin/bash

cargo build --release --target x86_64-unknown-linux-musl
mkdir ../bin &> /dev/null
cp ./target/release/powertools-rs ../bin/backend
