use rand::Rng; // To access random number generation
use std::process;

fn main() {
    // Define your possible exit codes
    let exit_codes = [1000, 1001, 1002];

    // Create a random number generator
    let mut rng = rand::thread_rng();

    // Select a random exit code from the list
    let random_exit_code = exit_codes[rng.gen_range(0..exit_codes.len())];

    println!("Exiting with code: {}", random_exit_code);

    // Exit with the randomly selected code
    process::exit(random_exit_code);
}
