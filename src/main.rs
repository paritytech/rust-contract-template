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
    let mut input = [0u8; 36];

    // store input data into `buffer`
    // it will trap if the buffer is smaller than the input
    api::call_data_copy(&mut input, 0);

    // the actual 4 byte integer is stored at offset 32
    // 4 byte selector
    // 28 byte padding as every integer is padded to be 32 byte
    let n = u32::from_be_bytes((&input[32..]).try_into().unwrap());

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
