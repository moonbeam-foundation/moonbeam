#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime_interface::runtime_interface;

#[runtime_interface]
pub trait MoonbeamExt {
    fn foo() {
        println!("Called from the runtime!");
    }
}