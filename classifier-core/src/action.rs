use brontes_tracer::types::CallFrameInfo;

use crate::context::DataContext;
use std::fmt::Debug;

pub trait ActionCollection: Default + Sync + Send {
    type DispatchOut;
    type ProtocolContext;

    fn dispatch<DB: DataContext<Self::ProtocolContext>>(
        &self,
        call_info: CallFrameInfo<'_>,
        db_ctx: &DB,
        block: u64,
        tx_idx: u64,
    ) -> Option<Self::DispatchOut>;
}

pub trait IntoAction: Debug + Send + Sync {
    type DecodeOut;
    type ProtocolContext;

    fn decode_call_trace<DB: DataContext<Self::ProtocolContext>>(
        &self,
        call_info: CallFrameInfo<'_>,
        block: u64,
        tx_idx: u64,
        db_ctx: &DB,
    ) -> eyre::Result<Self::DecodeOut>;
}
