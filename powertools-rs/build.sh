#!/bin/bash

cargo build --release
mkdir ../bin
# TODO replace "backend" \/ with binary name
cp ./target/release/backend ../bin/backend
