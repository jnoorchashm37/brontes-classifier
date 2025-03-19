pub mod action;
pub mod context;
pub mod types;

use action::ActionCollection;
use alloy_primitives::Log;
use alloy_rpc_types_trace::parity::Action;
use alloy_rpc_types_trace::parity::CallType;
pub use brontes_classifier_macros::{action_dispatch, action_impl};
use brontes_tracer::TracingClient;
use brontes_tracer::types::TransactionTraceWithLogs;
use brontes_tracer::types::TxTrace;
use context::DataContext;
use types::ClassifiedBlock;
use types::ClassifiedTrace;
use types::ClassifiedTx;
use types::collect_delegated_traces;

#[allow(async_fn_in_trait)]
pub trait TraceClassifier<A: ActionCollection> {
    type DataProvider: DataContext<A::ProtocolContext>;

    fn data_provider(&self) -> &Self::DataProvider;

    fn eth_provider(&self) -> &TracingClient;

    async fn classify_block(
        &self,
        block_number: u64,
    ) -> eyre::Result<ClassifiedBlock<A::DispatchOut>> {
        let tracer = self.eth_provider();
        let Some(tx_traces) = tracer
            .replay_block_transactions_with_inspector(block_number.into())
            .await?
        else {
            return Ok(ClassifiedBlock {
                block_number,
                transactions: Vec::new(),
            });
        };

        let transactions = tx_traces
            .into_iter()
            .enumerate()
            .map(|(tx_idx, tx_trace)| {
                self.classify_transaction(block_number, tx_idx as u64, tx_trace)
            })
            .collect::<Vec<_>>();

        Ok(ClassifiedBlock {
            block_number,
            transactions,
        })
    }

    fn classify_transaction(
        &self,
        block_number: u64,
        tx_idx: u64,
        trace: TxTrace,
    ) -> ClassifiedTx<A::DispatchOut> {
        let tx_hash = trace.tx_hash;

        let inner_traces = trace
            .trace
            .iter()
            .enumerate()
            .map(|(trace_idx, inner_trace)| {
                let classified_data = self.classify_transaction_trace(
                    block_number,
                    tx_idx as u64,
                    inner_trace.clone(),
                    &trace.trace,
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

        A::default().dispatch(call_info, self.data_provider(), block_number, tx_idx)
    }
}
