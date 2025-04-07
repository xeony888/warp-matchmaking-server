#!/bin/bash
source $HOME/.cargo/bin
git pull && cargo build --release && sudo systemctl restart warp-server && echo "Deployment complete"