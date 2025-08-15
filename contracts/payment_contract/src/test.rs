#![cfg(test)]

use core::arch::aarch64::uint8x8_t;

use super::*;
use soroban_sdk::{vec, Env, String};

fn add(a: u8, b: u8) -> u8 {
    a + b
}

#[test]
fn test() {
    let result = add(4, 6);
    assert_eq!(result, 10);
}
