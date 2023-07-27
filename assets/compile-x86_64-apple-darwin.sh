#!/bin/bash
set -e

export BASE_DIR=`pwd`
if [ ! -e dist ]; then mkdir dist; fi
# this may be useful: https://www.linkedin.com/pulse/rust-makes-cross-compilation-childs-play-marco-ieni/
# Intended for Github Actions
# Requires MacOS [x86, aarch64], rustup and nodejs, pnpm installed
# sudo apt update
# sudo apt install -y autogen libasound2-dev pkg-config make libssl-dev gcc g++ curl wget git libwebkit2gtk-4.0-dev
# Compile UI
cd $BASE_DIR/client
pnpm i
pnpm run build
cd $BASE_DIR
# Compile for linux
# cargo build --release
# strip target/release/onetagger
# strip target/release/onetagger-cli
# tar zcf OneTagger-linux.tar.gz -C target/release onetagger
# tar zcf OneTagger-linux-cli.tar.gz -C target/release onetagger-cli
# mv OneTagger-linux.tar.gz dist/
# mv OneTagger-linux-cli.tar.gz dist/
# Mac
# rustup target add x86_64-apple-darwin
# Install osxcross
# git clone https://github.com/tpoechtrager/osxcross
# cd osxcross
# wget -nc https://s3.dockerproject.org/darwin/v2/MacOSX10.10.sdk.tar.xz
# mv MacOSX10.10.sdk.tar.xz tarballs/
# UNATTENDED=yes OSX_VERSION_MIN=10.7 ./build.sh
# Set variables
export PATH="$PATH:$(pwd)/target/bin"
# export CC=o64-clang
# export CXX=o64-clang++
export MACOSX_DEPLOYMENT_TARGET=10.8
# export PKG_CONFIG_ALLOW_CROSS=1
# export PKG_CONFIG_PATH="$PATH:$(pwd)/target/SDK/MacOSX10.10.sdk/usr/lib/pkgconfig"
# cd $BASE_DIR
# Use cross preset
# mv .cargo/config.toml .cargo/config.toml.bak
# cp assets/mac-cross.toml .cargo/config.toml
# Compile 1t
cargo install cargo-bundle
cd crates/onetagger
cargo bundle --target x86_64-apple-darwin --release
cd $BASE_DIR/crates
cargo build --target x86_64-apple-darwin --release --bin onetagger-cli
# x86_64-apple-darwin14-strip target/x86_64-apple-darwin/release/onetagger
# x86_64-apple-darwin14-strip target/x86_64-apple-darwin/release/onetagger-cli
# Create own zip with proper permissions
cd $BASE_DIR/target/x86_64-apple-darwin/release/bundle/osx
# x86_64-apple-darwin14-strip OneTagger.app/Contents/MacOS/onetagger
chmod +x OneTagger.app/Contents/MacOS/onetagger
zip -r OneTagger-mac.zip .
cd -
mv $BASE_DIR/target/x86_64-apple-darwin/release/bundle/osx/OneTagger-mac.zip $BASE_DIR/dist/
# CLI
cd $BASE_DIR/target/x86_64-apple-darwin/release
chmod +x onetagger-cli
zip OneTagger-mac-cli.zip onetagger-cli
cd -
mv $BASE_DIR/target/x86_64-apple-darwin/release/OneTagger-mac-cli.zip $BASE_DIR/dist/