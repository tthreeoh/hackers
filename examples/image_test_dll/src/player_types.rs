use abi_stable::StableAbi;
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, StableAbi)]
pub enum JumpStyle {
    Normal = 0,
    Charge = 1,
}

impl Default for JumpStyle {
    fn default() -> Self {
        JumpStyle::Normal
    }
}
