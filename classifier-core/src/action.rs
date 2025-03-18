use alloy_primitives::FixedBytes;

use crate::{context::DataContext, types::CallFrameInfo};
use std::fmt::Debug;

pub trait ActionCollection<T>: Sync + Send {
    fn dispatch<DB: DataContext<P>, P>(
        &self,
        call_info: CallFrameInfo<'_>,
        db_tx: &DB,
        block: u64,
        tx_idx: u64,
    ) -> Option<T>;
}

pub trait IntoAction<T>: Debug + Send + Sync {
    fn decode_call_trace<DB: DataContext<P>, P>(
        &self,
        call_info: CallFrameInfo<'_>,
        block: u64,
        tx_idx: u64,
        db_tx: &DB,
    ) -> eyre::Result<T>;
}
