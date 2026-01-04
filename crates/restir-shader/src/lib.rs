#![no_std]
// allows `debug_printf!()` to be used in #[gpu_only] context
#![cfg_attr(target_arch = "spirv", feature(asm_experimental_arch))]
// otherwise you won't see any warnings
#![deny(warnings)]

pub mod camera;
pub mod material;
pub mod utils;
pub mod visibility;
