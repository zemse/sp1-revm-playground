//! A simple program to be proven inside the zkVM.

#![no_main]
sp1_zkvm::entrypoint!(main);

use tiny_keccak::{Hasher, Keccak};

// 16 - 11.32 - per sec 1.41;
// 32 - 18.15 - per sec 1.76
// 64 - 31.53 - per sec 2.03; 7.41 - per sec 8.63
// 128 - 62 - per sec 2.06
// 256 - 77 - per sec 3.32
// 512 - 139 - per sec 3.67; 15.40 - per sec 33.24
// 1024 - 289 - per sec 3.54; 26.71 - per sec 38.33

use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{address, keccak256, AccountInfo, Bytes, ExecutionResult, TransactTo, U256},
    Evm,
};

pub fn main() {
    let input = sp1_zkvm::io::read_vec();

    let input_code = bincode::deserialize::<Vec<u8>>(&input).unwrap();
    let input_code = Bytes::from(input_code);
    println!("input code: {:?}", input_code);

    let evm = Evm::builder();
    let dummy_address = address!("1234567812345678123456781234567812345678");
    let db = CacheDB::new(EmptyDB::new());
    let evm = evm
        .with_db(db)
        .modify_db(|db| {
            db.insert_account_info(
                dummy_address,
                AccountInfo {
                    balance: U256::ZERO,
                    nonce: 1,
                    code_hash: keccak256(&input_code),
                    code: Some(revm::primitives::Bytecode::new_raw(input_code)),
                },
            )
        })
        .modify_tx_env(|tx| {
            tx.transact_to = TransactTo::Call(dummy_address);
        });

    let mut evm = evm.build();

    let result = evm.transact_commit().unwrap();

    let value = print_exec_result(result, 0);
    sp1_zkvm::io::commit(&value);
    // for _ in 0..4 {
    //     let mut hasher = Keccak::v256();
    //     hasher.update(&value);
    //     hasher.finalize(&mut value);
    // }
    // sp1_zkvm::io::commit(&value);
    // NOTE: values of n larger than 186 will overflow the u128 type,
    // resulting in output that doesn't match fibonacci sequence.
    // However, the resulting proof will still be valid!
    // let n = sp1_zkvm::io::read::<u32>();
    // let mut a: u128 = 0;
    // let mut b: u128 = 1;
    // let mut sum: u128;
    // // sum = a + b;
    // for _ in 1..80000 {
    //     sum = a + b;
    //     a = b;
    //     b = sum;
    // }

    // sp1_zkvm::io::commit(&a);
    // sp1_zkvm::io::commit(&b);
}

fn print_exec_result(result: ExecutionResult, initial_gas_spend: u64) -> u64 {
    let gas_used = match result {
        ExecutionResult::Success {
            gas_used, output, ..
        } => {
            let data = match output {
                revm::primitives::Output::Call(data) => data,
                revm::primitives::Output::Create(_, _) => unreachable!(),
            };
            println!("Success!\nReturndata: {data}");
            gas_used
        }
        ExecutionResult::Revert { gas_used, output } => {
            println!("Revert!\nRevertdata: {output}");
            gas_used
        }
        ExecutionResult::Halt { reason, gas_used } => {
            println!("Halt!\nReason: {reason:?}");
            gas_used
        }
    };
    gas_used - initial_gas_spend
}
