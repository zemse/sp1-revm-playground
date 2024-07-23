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
    inspector_handle_register,
    interpreter::{Interpreter, OpCode},
    primitives::{
        address, bytes, keccak256, AccountInfo, Bytes, ExecutionResult, HashMap, TransactTo, U256,
    },
    Database, Evm, EvmContext, Inspector,
};

struct Logger;

impl<DB: Database> Inspector<DB> for Logger {
    fn step(&mut self, interp: &mut Interpreter, _context: &mut EvmContext<DB>) {
        let opcode_num = interp.current_opcode();
        let some_opcode = OpCode::new(opcode_num);
        let opcode_str = some_opcode.map(|op| op.as_str()).unwrap_or("UNKNOWN");
        println!("op start {}", opcode_str);
    }

    // fn step_end(&mut self, interp: &mut Interpreter, _context: &mut EvmContext<DB>) {
    //     let opcode_num = interp.current_opcode();
    //     let some_opcode = OpCode::new(opcode_num);
    //     let opcode_str = some_opcode.map(|op| op.as_str()).unwrap_or("UNKNOWN");
    //     println!("op end {}", opcode_str);
    // }
}

pub fn main() {
    println!("hey 123");
    let input = sp1_zkvm::io::read_vec();

    let input_code = bincode::deserialize::<Vec<u8>>(&input).unwrap();
    // let input_code: Vec<u8> = bytes!("5f505f505f505f505f505f505f505f505f505f50").to_vec();
    let input_code = Bytes::from(input_code);
    println!("input code: {:?}", input_code);

    let evm = Evm::builder();
    let dummy_address = address!("1234567812345678123456781234567812345678");
    let db = CacheDB::new(EmptyDB::new());
    let evm = evm
        .with_db(db)
        // .with_external_context(Logger)
        // .append_handler_register(inspector_handle_register)
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
}

fn print_exec_result(result: ExecutionResult, initial_gas_spend: u64) -> u64 {
    let gas_used = match result {
        ExecutionResult::Success {
            gas_used, output, ..
        } => {
            // let data = match output {
            //     revm::primitives::Output::Call(data) => data,
            //     revm::primitives::Output::Create(_, _) => unreachable!(),
            // };
            // println!("Success!\nReturndata: {data}");
            gas_used
        }
        ExecutionResult::Revert { gas_used, output } => {
            // println!("Revert!\nRevertdata: {output}");
            gas_used
        }
        ExecutionResult::Halt { reason, gas_used } => {
            // println!("Halt!\nReason: {reason:?}");
            gas_used
        }
    };
    gas_used - initial_gas_spend
}
