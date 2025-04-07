#!/bin/bash

cd .

git pull origin main

cargo build --release

sudo systemctl restart your-service-name

echo "Deployment complete"