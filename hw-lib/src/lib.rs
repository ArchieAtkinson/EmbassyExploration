#![no_std]
#![allow(async_fn_in_trait)]

use panic_probe as _;

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
