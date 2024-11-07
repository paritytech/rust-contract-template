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
    let mut buffer: [u8; 512] = [0; 512];
    let mut input = buffer.as_mut();

    // store input data into `buffer`
    // it will trap if the buffer is smaller than the input
    api::input(&mut input);

    // the reference is resized to the actual length
    let input_len = input.len() as u32;

    // emitting a log is helpful for debugging
    api::deposit_event(&[], input);

    // return the length to the caller
    api::return_value(ReturnFlags::empty(), &input_len.to_be_bytes());
}
