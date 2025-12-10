#![no_main]
#![no_std]

use alloy_core::{
    sol,
    sol_types::{SolCall, SolValue},
};
use uapi::{HostFn, HostFnImpl as api, ReturnFlags};

extern crate alloc;
use alloc::vec;

#[global_allocator]
static mut ALLOC: picoalloc::Mutex<picoalloc::Allocator<picoalloc::ArrayPointer<1024>>> = {
    static mut ARRAY: picoalloc::Array<1024> = picoalloc::Array([0u8; 1024]);

    picoalloc::Mutex::new(picoalloc::Allocator::new(unsafe {
        picoalloc::ArrayPointer::new(&raw mut ARRAY)
    }))
};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // Safety: The unimp instruction is guaranteed to trap
    unsafe {
        core::arch::asm!("unimp");
        core::hint::unreachable_unchecked();
    }
}

/// This is the constructor which is called once per contract.
#[no_mangle]
#[polkavm_derive::polkavm_export]
pub extern "C" fn deploy() {}

// The contract use the following interface
sol!(
    interface IRustContract {
        function fibonacci(uint32) external pure returns (uint32);
        function extcodecopyOp(
        address account,
        uint256 offset,
        uint256 size
    ) public pure returns (bytes memory code);

    }
);

/// Decode input using the sol! macro
fn decode_input() -> u32 {
    let call_data_len = api::call_data_size();
    let mut call_data = vec![0u8; call_data_len as usize];
    api::call_data_copy(&mut call_data, 0);

    // Decode the input using the generated struct
    let decoded =
        IRustContract::fibonacciCall::abi_decode(&call_data, true).expect("Failed to decode input");

    decoded._0
}

/// Encode output
fn encode_output(result: u32) -> vec::Vec<u8> {
    result.abi_encode()
}

#[allow(dead_code)]
fn decode_input_manual() -> u32 {
    // function fibonacci(uint32) external pure returns (uint32);

    // ❯ cast calldata "fibonnaci(uint) view returns(uint)" "42" | xxd -r -p | xxd -c 32 -g 1
    //00000000: 50 7a 10 34 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
    //00000020: 00 00 00 2a

    // The input is abi encoded as follows:
    // - 4 byte selector
    // - 32 byte padded integer

    // the actual 4 byte integer is stored at offset 32
    let mut input = [0u8; 4];
    api::call_data_copy(&mut input, 32);

    u32::from_be_bytes(input)
}

#[allow(dead_code)]
fn encode_output_manual(result: u32) -> [u8; 32] {
    // pad the result to 32 byte
    let mut output = [0u8; 32];
    output[28..].copy_from_slice(&result.to_be_bytes());
    output
}

/// This is the regular entry point when the contract is called.
#[no_mangle]
#[polkavm_derive::polkavm_export]
pub extern "C" fn call() {
    // Use the new sol! macro approach
    let n = decode_input();
    let result = fibonacci(n);
    let encoded_output = encode_output(result);

    // Return the encoded output
    api::return_value(ReturnFlags::empty(), &encoded_output);
}

fn fibonacci(n: u32) -> u32 {
    if n == 0 {
        0
    } else if n == 1 {
        1
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}
