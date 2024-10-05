include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::CString;

pub fn mine_block_rust(block_data: &str, difficulty: i32, max_nonce: i32) -> Option<(String, i32)> {
    // Convert Rust string to C-compatible string
    let block_data_c = CString::new(block_data).unwrap();

    // Prepare buffers for the result
    let mut final_hash = [0u8; 32];
    let mut final_nonce = 0;

    // Call the C++ function from Rust using FFI
    unsafe {
        mine_block(
            block_data_c.as_ptr(),
            block_data.len() as i32,
            difficulty,
            final_hash.as_mut_ptr(),
            &mut final_nonce,
            max_nonce,
        );
    }

    // Convert result into a Rust-compatible format
    if final_nonce != -1 {
        Some((hex::encode(final_hash), final_nonce))
    } else {
        None
    }
}
