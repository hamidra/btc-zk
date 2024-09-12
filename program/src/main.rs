// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);
use sha2::{Digest, Sha256};

pub fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a custom system call which handles reading inputs
    // from the prover.
    let prev_hash_input = sp1_zkvm::io::read::<Vec<u8>>();
    let header_bytes = sp1_zkvm::io::read::<Vec<u8>>();

    let prev_hash = &header_bytes[4..36];

    // block hash (double sha2)
    let digest = Sha256::digest(&header_bytes);
    let digest_digest = Sha256::digest(&digest);

    let hash_is_valid = prev_hash == prev_hash_input;

    // To_Do: check block difficulty

    sp1_zkvm::io::commit(&hash_is_valid);
}
