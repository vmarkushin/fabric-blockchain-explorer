# Fabirc Blockchain Explorer

## Overview
This is a simple HyperLedger Fabric blockchain explorer written on Rust.
All information is presented in a web interface. 
Currently, the explorer has the following features:
- Fetching channel information
- Fetching information about blocks and transactions.

## Installation
1. Install the latest version of Fabric: `curl -sSL https://bit.ly/2ysbOFE | bash -s`
1. Go to test project: `cd fabric-samples/test-network`
1. Run the network: `network.sh up createChannel`
1. Set env-var for the explorer `export FABRIC_PROJ_PATH=$PWD`
1. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
1. Run explorer: `cargo run`
1. Open `http://127.0.0.1:3030/blocks` in browser
