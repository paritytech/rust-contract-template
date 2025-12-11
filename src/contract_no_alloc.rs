#![no_main]
#![no_std]

use pallet_revive_uapi::{HostFn, HostFnImpl as api, ReturnFlags, StorageFlags};
//
// Function selectors
const TRANSFER_SELECTOR: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb]; // transfer(address,uint256)
const MINT_SELECTOR: [u8; 4] = [0x40, 0xc1, 0x0f, 0x19]; // mint(address,uint256)

// Event signature hash for Transfer(address,address,uint256)
const TRANSFER_EVENT_SIGNATURE: [u8; 32] = [
    0xdd, 0xf2, 0x52, 0xad, 0x1b, 0xe2, 0xc8, 0x9b, 0x69, 0xc2, 0xb0, 0x68, 0xfc, 0x37, 0x8d, 0xaa,
    0x95, 0x2b, 0xa7, 0xf1, 0x63, 0xc4, 0xa1, 0x16, 0x28, 0xf5, 0x5a, 0x4d, 0xf5, 0x23, 0xb3, 0xef,
];

// Error selector for InsufficientBalance()
const INSUFFICIENT_BALANCE_ERROR: [u8; 4] = [0xf4, 0xd6, 0x78, 0xb8];

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // Safety: The unimp instruction is guaranteed to trap
    unsafe {
        core::arch::asm!("unimp");
        core::hint::unreachable_unchecked();
    }
}

/// Storage key for totalSupply (slot 0)
#[inline]
fn total_supply_key() -> [u8; 32] {
    [0u8; 32] // Slot 0
}

/// Helper function to compute storage key for balances[address]
/// Storage slot for balances mapping is 1 (totalSupply is at slot 0)
/// Follows Solidity convention: keccak256(leftPad32(key) ++ leftPad32(slot))
fn balance_key(addr: &[u8; 20]) -> [u8; 32] {
    let mut input = [0u8; 64]; // 32 bytes (padded address) + 32 bytes (slot)

    // First 32 bytes: address left-padded to 32 bytes (12 zeros + 20 address bytes)
    input[12..32].copy_from_slice(addr);

    // Last 32 bytes: slot 1 for balances mapping (slot 0 is totalSupply)
    input[63] = 1;

    let mut key = [0u8; 32];
    api::hash_keccak_256(&input, &mut key);
    key
}

/// Get totalSupply from storage
fn get_total_supply() -> u128 {
    let key = total_supply_key();
    let mut supply_bytes = [0u8; 16];
    let mut supply_slice = &mut supply_bytes[..];

    match api::get_storage(StorageFlags::empty(), &key, &mut supply_slice) {
        Ok(_) => u128::from_be_bytes(supply_bytes),
        Err(_) => 0u128,
    }
}

// #[inline(always)]
fn to_word(v: u128) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[16..].copy_from_slice(&v.to_be_bytes());
    out
}

/// Set totalSupply in storage
#[inline]
fn set_total_supply(amount: u128) {
    let key = total_supply_key();
    let bytes = amount.to_be_bytes();
    api::set_storage(StorageFlags::empty(), &key, &bytes);
}

/// Get the balance for a given address from storage
#[inline]
fn get_balance(addr: &[u8; 20]) -> u128 {
    let key = balance_key(addr);
    let mut balance_bytes = [0u8; 16];
    let mut balance_slice = &mut balance_bytes[..];

    match api::get_storage(StorageFlags::empty(), &key, &mut balance_slice) {
        Ok(_) => u128::from_be_bytes(balance_bytes),
        Err(_) => 0u128,
    }
}

/// Set the balance for a given address in storage
#[inline]
fn set_balance(addr: &[u8; 20], amount: u128) {
    let key = balance_key(addr);
    let bytes = amount.to_be_bytes();
    api::set_storage(StorageFlags::empty(), &key, &bytes);
}

/// Emit a Transfer event
#[inline]
fn emit_transfer(from: &[u8; 20], to: &[u8; 20], value: u128) {
    let mut from_topic = [0u8; 32];
    from_topic[12..32].copy_from_slice(from);

    let mut to_topic = [0u8; 32];
    to_topic[12..32].copy_from_slice(to);

    let topics = [TRANSFER_EVENT_SIGNATURE, from_topic, to_topic];
    let data = to_word(value);
    api::deposit_event(&topics, &data);
}

/// Revert with an InsufficientBalance error
#[inline]
fn revert_insufficient_balance() -> ! {
    api::return_value(ReturnFlags::REVERT, &INSUFFICIENT_BALANCE_ERROR);
}

/// Get the caller's address
#[inline]
fn get_caller() -> [u8; 20] {
    let mut caller = [0u8; 20];
    api::caller(&mut caller);
    caller
}

/// Decode address from ABI-encoded data (32 bytes, address is in the last 20 bytes)
#[inline]
fn decode_address(data: &[u8]) -> [u8; 20] {
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&data[12..32]);
    addr
}

/// Decode u128 from ABI-encoded data (32 bytes)
#[inline]
fn decode_u128(data: &[u8]) -> u128 {
    u128::from_be_bytes(data[16..32].try_into().unwrap())
}

/// This is the constructor which is called once per contract.
#[no_mangle]
#[polkavm_derive::polkavm_export]
pub extern "C" fn deploy() {}

/// This is the regular entry point when the contract is called.
#[no_mangle]
#[polkavm_derive::polkavm_export]
pub extern "C" fn call() {
    let call_data_len = api::call_data_size() as usize;

    // Fixed buffer for call data
    let mut call_data = [0u8; 256];
    if call_data_len > call_data.len() {
        panic!("Call data too large");
    }

    api::call_data_copy(&mut call_data[..call_data_len], 0);

    if call_data_len < 4 {
        panic!("Call data too short");
    }

    let selector: [u8; 4] = call_data[0..4].try_into().unwrap();

    match selector {
        TRANSFER_SELECTOR => {
            // ABI encoding: selector(4) + address(32) + uint256(32)
            if call_data_len < 68 {
                panic!("Invalid transfer call data");
            }

            let to = decode_address(&call_data[4..36]);
            let amount = decode_u128(&call_data[36..68]);

            let caller = get_caller();
            let sender_balance = get_balance(&caller);

            if sender_balance < amount {
                revert_insufficient_balance();
            }

            let new_sender_balance = sender_balance - amount;
            let recipient_balance = get_balance(&to);
            let new_recipient_balance = recipient_balance + amount;

            set_balance(&caller, new_sender_balance);
            set_balance(&to, new_recipient_balance);
            emit_transfer(&caller, &to, amount);
        }
        MINT_SELECTOR => {
            // ABI encoding: selector(4) + address(32) + uint256(32)
            if call_data_len < 68 {
                panic!("Invalid mint call data");
            }

            let to = decode_address(&call_data[4..36]);
            let amount = decode_u128(&call_data[36..68]);

            let new_recipient_balance = get_balance(&to).saturating_add(amount);
            set_balance(&to, new_recipient_balance);

            let new_supply = get_total_supply().saturating_add(amount);
            set_total_supply(new_supply);

            let zero_address = [0u8; 20];
            emit_transfer(&zero_address, &to, amount);
        }
        _ => panic!("Unknown function selector"),
    }
}
