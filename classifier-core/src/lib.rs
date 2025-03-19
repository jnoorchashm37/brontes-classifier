pub mod action;
pub mod context;
pub mod types;

use action::ActionCollection;
use alloy_primitives::Log;
use alloy_rpc_types_trace::parity::Action;
use alloy_rpc_types_trace::parity::CallType;
use alloy_rpc_types_trace::parity::TraceResultsWithTransactionHash;
pub use brontes_classifier_macros::{action_dispatch, action_impl};
use context::DataContext;
use types::ClassifiedBlock;
use types::ClassifiedTrace;
use types::ClassifiedTx;
use types::FullTransactionTraceWithLogs;
use types::TransactionTraceWithLogs;
use types::collect_delegated_traces;

pub trait TraceClassifier<A: ActionCollection> {
    type DataProvider: DataContext<A::ProtocolContext>;

    fn provider(&self) -> &Self::DataProvider;

    fn classify_block(
        &self,
        block_number: u64,
        block_trace: Vec<TraceResultsWithTransactionHash>,
    ) -> ClassifiedBlock<A::DispatchOut> {
        let transactions = block_trace
            .into_iter()
            .enumerate()
            .map(|(tx_idx, tx_trace)| {
                self.classify_transaction(block_number, tx_idx as u64, tx_trace)
            })
            .collect::<Vec<_>>();

        ClassifiedBlock {
            block_number,
            transactions,
        }
    }

    fn classify_transaction(
        &self,
        block_number: u64,
        tx_idx: u64,
        trace: TraceResultsWithTransactionHash,
    ) -> ClassifiedTx<A::DispatchOut> {
        let tx_hash = trace.transaction_hash;

        let full_traces = FullTransactionTraceWithLogs::new(trace, vec![]);
        let inner_traces = full_traces
            .tx_traces
            .iter()
            .enumerate()
            .map(|(trace_idx, inner_trace)| {
                let classified_data = self.classify_transaction_trace(
                    block_number,
                    tx_idx as u64,
                    inner_trace.clone(),
                    &full_traces.tx_traces,
                );

                ClassifiedTrace {
                    trace_idx: trace_idx as u64,
                    classified_data,
                    msg_sender: inner_trace.msg_sender,
                }
            })
            .collect::<Vec<_>>();

        ClassifiedTx {
            tx_hash,
            traces: inner_traces,
            tx_idx,
        }
    }

    fn classify_transaction_trace(
        &self,
        block_number: u64,
        tx_idx: u64,
        trace: TransactionTraceWithLogs,
        full_trace: &[TransactionTraceWithLogs],
    ) -> Option<A::DispatchOut> {
        if trace.is_static_call() {
            return None;
        }

        let mut call_info = trace.get_callframe_info();
        // Add logs of delegated calls to the root trace, only if the delegated call is
        // from the same address / in the same call frame.
        if let Action::Call(root_call) = &trace.trace.action {
            let mut delegated_traces = Vec::new();
            collect_delegated_traces(
                full_trace,
                &trace.trace.trace_address,
                &mut delegated_traces,
            );

            for delegated_trace in delegated_traces {
                if let Action::Call(delegated_call) = &delegated_trace.trace.action {
                    if let CallType::DelegateCall = delegated_call.call_type {
                        if delegated_call.from == root_call.to {
                            let logs_internal = delegated_trace.logs.iter().collect::<Vec<&Log>>();
                            call_info.delegate_logs.extend(logs_internal);
                        }
                    }
                }
            }
        }

        A::default().dispatch(call_info, self.provider(), block_number, tx_idx)
    }
}
