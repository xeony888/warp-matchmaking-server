

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
3.  