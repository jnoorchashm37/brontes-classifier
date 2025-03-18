use brontes_classifier::action_dispatch;
// pub mod erc20;
pub mod abis;

pub mod uniswap;
pub use uniswap::*;

action_dispatch!(
    (UniswapSwapClassifer, crate::classifier::Protocol) => Actions | UniswapV2SwapCall
);

#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    UniswapV2,
    UniswapV3,
}

// impl ProtocolContext for Protocol {
//     fn to_byte(&self) -> u8 {
//         *self as u8
//     }
// }

pub enum Actions {
    Swap(Swap),
}

pub struct Swap {}

