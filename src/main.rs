#![no_main]
#![no_std]

use uapi::{HostFn, HostFnImpl as api, ReturnFlags};

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

/// This is the regular entry point when the contract is called.
#[no_mangle]
#[polkavm_derive::polkavm_export]
pub extern "C" fn call() {
    // We want this contract to be called with the following ABI:
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

    // Note for more complex input, sol! macro can be used to encod and decode input and output
    // https://docs.rs/alloy-core/0.8.24/alloy_core/sol_types/macro.sol.html
    let n = u32::from_be_bytes(input);
    let result = fibonacci(n);

    // pad the result to 32 byte
    let mut output = [0u8; 32];
    output[28..].copy_from_slice(&result.to_be_bytes());

    // returning without calling this function leaves the output buffer empty
    api::return_value(ReturnFlags::empty(), &output);
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
