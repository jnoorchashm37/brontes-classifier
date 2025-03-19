use std::collections::HashMap;

use crate::classifier::uniswap_v2::*;
use crate::classifier::uniswap_v3::*;
use alloy_primitives::{Address, U256};

use brontes_classifier::action_dispatch;
use brontes_classifier::context::DataContext;

action_dispatch!(
    (UniswapSwapClassifer, Protocol) => Actions | UniswapV2SwapCall, UniswapV3SwapCall
);

#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    UniswapV2,
    UniswapV3,
}

#[derive(Debug, Clone)]
pub enum Actions {
    Swap(ActionSwap),
}

#[derive(Debug, Clone)]
pub struct ActionSwap {
    pub protocol: Protocol,
    pub pool: Address,
    pub recipient: Address,
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: U256,
    pub amount_out: U256,
}

#[derive(Debug, Clone)]
pub struct DataCache {
    pub cache: HashMap<Address, UniswapProtocolTokens>,
}

impl DataCache {
    pub fn new(values: Vec<(Address, UniswapProtocolTokens)>) -> Self {
        Self {
            cache: HashMap::from_iter(values),
        }
    }
}

impl DataContext<Protocol> for DataCache {
    fn get_protocol(&self, target_address: Address) -> eyre::Result<Protocol> {
        Ok(self
            .cache
            .get(&target_address)
            .ok_or(eyre::eyre!("protocol does not exist"))?
            .protocol)
    }

    fn get_protocol_tokens_sorted(&self, target_address: Address) -> eyre::Result<Vec<Address>> {
        Ok(self
            .cache
            .get(&target_address)
            .ok_or(eyre::eyre!("address could not be fetched"))?
            .sorted_tokens
            .to_vec())
    }
}

#[derive(Debug, Clone)]
pub struct UniswapProtocolTokens {
    pub protocol: Protocol,
    pub sorted_tokens: [Address; 2],
}
