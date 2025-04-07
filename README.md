

## to build dev simulation: 
cargo build --release --bin game-simulation

Instead of running the game, program will run this simulation program, which accepts ws connections and exits in 60 seconds


## Steps to setup and test
1. cargo build --release --bin game-simulation
2. npm i
3. cargo run
4. (in new terminal) npm run test

## Steps to setup for prod
1. cargo build --release
2. chmod +x (all game executables)
3. create .env file and populate it
4. sudo systemctl daemon-reload 
5. sudo systemctl start warp-server 
6. sudo systemctl enable warp-server


## Useful tools
- sudo lsof -i -P -n // see which processes are running and what ports they are running on (can be used to view game processes)
- sudo systemctl status warp-server // restart server daemon
- sudo lsof -i :8080 // what processes are running on port 8080

## Pull new commit from github and run it
1. chmod +x ./reload.sh
2. ./reload.sh // automatically performs all tasks below

## reload flow
1. git pull
2. sudo systemctl stop warp-server
3. cargo build --release
4. sudo systemctl start warp-server
5. sudo systemctl enable warp-server

## Paths
- executable ./target/release/rust-matchmaking-server
- 