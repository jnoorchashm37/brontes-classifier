use alloy_primitives::{Address, TxHash};

use brontes_tracer::types::TransactionTraceWithLogs;
pub use brontes_tracer::types::{CallFrameInfo, CallInfo};

#[derive(Debug, Clone)]
pub struct ClassifiedBlock<A> {
    pub block_number: u64,
    pub transactions: Vec<ClassifiedTx<A>>,
}

#[derive(Debug, Clone)]
pub struct ClassifiedTx<A> {
    pub tx_hash: TxHash,
    pub tx_idx: u64,
    pub traces: Vec<ClassifiedTrace<A>>,
}

#[derive(Debug, Clone)]
pub struct ClassifiedTrace<A> {
    pub classified_data: Option<A>,
    pub trace_idx: u64,
    pub msg_sender: Address,
}

pub fn collect_delegated_traces<'a>(
    traces: &'a [TransactionTraceWithLogs],
    parent_trace_address: &[usize],
    delegated_traces: &mut Vec<&'a TransactionTraceWithLogs>,
) {
    for trace in traces {
        let subtrace_address = &trace.trace.trace_address;
        if subtrace_address.starts_with(parent_trace_address)
            && subtrace_address.len() == parent_trace_address.len() + 1
        {
            delegated_traces.push(trace);
            collect_delegated_traces(traces, subtrace_address, delegated_traces);
        }
    }
}
