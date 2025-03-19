use crate::abis::UniswapV3;
use brontes_classifier::action_impl;
use brontes_classifier::types::CallInfo;

use crate::types::ActionSwap;
use crate::types::Actions;
use crate::types::Protocol;

action_impl! {
    (Protocol, Actions),
    Protocol::UniswapV3,
    UniswapV3::swapCall,
    Swap,
    [Swap],
    call_data: true,
    return_data: true,
    |
    info: CallInfo,
    call_data: swapCall,
    return_data: swapReturn,
    db_tx: &DB| {
        let token_0_delta = return_data.amount0.abs().try_into().unwrap();
        let token_1_delta = return_data.amount1.abs().try_into().unwrap();
        let recipient = call_data.recipient;
        let tokens = db_tx.get_protocol_tokens_sorted(info.target_address)?;

        let (token_in, amount_in, token_out, amount_out) = if return_data.amount0.is_negative() {
            (tokens[1], token_1_delta, tokens[0], token_0_delta)
        } else {
            (tokens[0], token_0_delta, tokens[1], token_1_delta)
        };

        Ok(ActionSwap {
            protocol: Protocol::UniswapV2,
            pool: info.target_address,
            recipient,
            token_in,
            token_out,
            amount_in,
            amount_out,

        })
    }
}
