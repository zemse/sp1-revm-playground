// ! A simple script to generate and verify the proof of a given program.

use revm::primitives::bytes;
use sp1_core::{
    runtime::{Program, Runtime},
    utils::SP1CoreOpts,
};
use sp1_sdk::{ProverClient, SP1Stdin};

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

#[allow(unreachable_code)]
fn main() {
    println!("running script");
    // Generate proof.
    let mut stdin = SP1Stdin::new();
    // let n = 2u32;

    // Input Bytecode to execute on EVM
    let n: Vec<u8> = bytes!("5f5f5f5f").to_vec();

    stdin.write(&n);

    let program = Program::from(ELF);
    let opts = SP1CoreOpts::default();
    let mut runtime = Runtime::new(program, opts);
    runtime.write_vecs(&stdin.buffer);
    runtime.run_untraced().unwrap();
    // runtime.counter.end(runtime.steps_counter);
    println!("counter {:?}", runtime.risc5_counter);

    return;

    let client = ProverClient::new();

    use std::time::Instant;
    let setup_time = Instant::now();
    let (pk, vk) = client.setup(ELF);
    println!("Setup time: {:.2?}", setup_time.elapsed());

    let prove_time = Instant::now();
    let mut proof = client.prove_compressed(&pk, stdin).expect("proving failed");
    println!("Proof time: {:.2?}", prove_time.elapsed());

    // Read output.
    let a = proof.public_values.read::<u128>();
    let b = proof.public_values.read::<u128>();
    println!("a: {}", a);
    println!("b: {}", b);

    // Verify proof.
    client
        .verify_compressed(&proof, &vk)
        .expect("verification failed");

    // Save proof.
    proof
        .save("proof-with-io.json")
        .expect("saving proof failed");

    println!("successfully generated and verified proof for the program!")
}
