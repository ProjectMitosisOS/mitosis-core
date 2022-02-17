#!/bin/bash

# verify rust installed or not
if rustc --version; then
    echo "rust is already installed"
else
    echo "install rust now"
    curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh -s -- -y
    source $HOME/.cargo/env
fi

rustup default nightly-2022-02-05-x86_64-unknown-linux-gnu
rustup component add rust-src
