# !/bin/bash

project_name=chip8-rust

# exit on error
set -e

rustup target add wasm32-unknown-unknown
./wasm-bindgen-macroquad.sh --release $project_name
rm -rf github_pages

# copy assets
cp -r assets dist/

# put it into a folder that will be committed and pushed
cp -r dist github_pages
