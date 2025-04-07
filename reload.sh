#!/bin/bash

cd .

git pull origin main

cargo build --release

sudo systemctl restart warp-server

echo "Deployment complete"