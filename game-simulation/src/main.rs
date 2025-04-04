use rand::Rng; // To access random number generation
use std::process;

fn main() {
    let exit_codes = [1000, 1001, 1002];

    let mut rng = rand::thread_rng();

    let random_exit_code = exit_codes[rng.gen_range(0..exit_codes.len())];

    println!("Exiting with code: {}", random_exit_code);

    process::exit(random_exit_code);
}
