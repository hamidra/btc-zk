//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be verified
//! on-chain.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --package fibonacci-script --bin prove --release
//! ```

use std::path::PathBuf;

use alloy_sol_types::{sol, SolType};
use bitcoin::block_data::{test_json, BlockReader};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sp1_sdk::{HashableKey, ProverClient, SP1PlonkBn254Proof, SP1Stdin, SP1VerifyingKey};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
///
/// This file is generated by running `cargo prove build` inside the `program` directory.
pub const ELF: &[u8] = include_bytes!("../../../program/elf/riscv32im-succinct-zkvm-elf");

/// The public values encoded as a tuple that can be easily deserialized inside Solidity.
type PublicValuesTuple = sol! {
    tuple(bool,)
};

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the program.
    let (pk, vk) = client.setup(ELF);

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();

    // Read blocks
    let block_reader =
        bitcoin::block_data::BlockReader::new_from_json(test_json::TEST_JSON_RPC).unwrap();
    let headers = block_reader.get_block_headers().unwrap();
    if (headers.len() < 2) {
        println!("no block header was loaded!");
        return;
    }
    let header_digest = Sha256::digest(headers[1].1.to_bytes()).as_slice().to_vec();
    let prev_hash = Sha256::digest(header_digest).as_slice().to_vec();
    stdin.write(&prev_hash);

    let block_header = headers[2].1.to_bytes();
    stdin.write(&block_header);

    // Generate the proof.
    let mut proof = client.prove(&pk, stdin).expect("failed to generate proof");
    println!("proof: {:?}", proof);
    let is_valid = proof.public_values.read::<bool>();

    println!("Successfully generated proof!");
    println!("bitcoin header validation: {:?}", is_valid);

    // Verify the proof.
    client.verify(&proof, &vk).expect("failed to verify proof");
}
