use alloy_primitives::U256;
use brontes_classifier::action_impl;
use brontes_classifier::types::CallInfo;

use crate::abis::UniswapV2;
use crate::types::ActionSwap;
use crate::types::Actions;
use crate::types::Protocol;

action_impl! {
    (Protocol, Actions),
    crate::types::Protocol::UniswapV2,
    UniswapV2::swapCall,
    Swap,
    [..Swap],
    call_data: true,
    logs: true,
    |
    info: CallInfo,
    call_data: swapCall,
    log_data: UniswapV2SwapCallLogs,
    db_ctx: &DB| {
        let logs = log_data.swap_field?;
        let recipient = call_data.to;

        let tokens = db_ctx.get_protocol_tokens_sorted(info.target_address)?;

        let (token_in, amount_in, token_out, amount_out) = if logs.amount0In == U256::ZERO {
            (tokens[1], logs.amount1In, tokens[0], logs.amount0In)
        } else {
            (tokens[0], logs.amount0In, tokens[1], logs.amount1In)
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
