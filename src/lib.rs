#![cfg_attr(not(test), no_std)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

#[cfg(test)]
mod test;

pub mod hl;
pub mod ll;
