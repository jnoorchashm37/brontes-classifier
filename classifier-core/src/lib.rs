pub mod action;
pub mod context;
pub mod types;

use action::ActionCollection;
use alloy_primitives::Log;
use alloy_rpc_types_trace::parity::Action;
use alloy_rpc_types_trace::parity::CallType;
pub use brontes_classifier_macros::{action_dispatch, action_impl};
use context::DataContext;
use types::TransactionTraceWithLogs;
use types::collect_delegated_traces;

pub trait TraceClassifier<A: ActionCollection> {
    type DataProvider: DataContext<A::ProtocolContext>;

    fn provider(&self) -> &Self::DataProvider;

    fn classify_call(
        &self,
        block: u64,
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

        A::default().dispatch(call_info, self.provider(), block, tx_idx)
    }
}
