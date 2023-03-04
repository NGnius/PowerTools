#!/bin/bash

git clone https://github.com/NGnius/usdpl-rs usdpl-rs
cd usdpl-rs/usdpl-front/

./build.sh $1 $2

cd ../..

cp -f ./usdpl-rs/usdpl-front/pkg/* ./
#rm ./.gitignore

rm -rf ./usdpl-rs
