use alloy_primitives::{Address, Bytes, Log, TxHash, U256};
use alloy_rpc_types_trace::parity::{
    Action, CallType, TraceResultsWithTransactionHash, TransactionTrace,
};

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

#[derive(Debug, Clone)]
pub struct CallFrameInfo<'a> {
    pub trace_idx: u64,
    pub call_data: Bytes,
    pub return_data: Bytes,
    pub target_address: Address,
    pub from_address: Address,
    pub logs: &'a [Log],
    pub delegate_logs: Vec<&'a Log>,
    pub msg_sender: Address,
    pub msg_value: U256,
}

impl CallFrameInfo<'_> {
    pub fn get_fixed_fields(&self) -> CallInfo {
        CallInfo {
            trace_idx: self.trace_idx,
            target_address: self.target_address,
            from_address: self.from_address,
            msg_sender: self.msg_sender,
            msg_value: self.msg_value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CallInfo {
    pub trace_idx: u64,
    pub target_address: Address,
    pub from_address: Address,
    pub msg_sender: Address,
    pub msg_value: U256,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullTransactionTraceWithLogs {
    pub tx_hash: TxHash,
    pub tx_traces: Vec<TransactionTraceWithLogs>,
}

impl FullTransactionTraceWithLogs {
    pub fn new(trace_results: TraceResultsWithTransactionHash, logs: Vec<Log>) -> Self {
        if trace_results.full_trace.trace.is_empty() {
            return FullTransactionTraceWithLogs {
                tx_hash: trace_results.transaction_hash,
                tx_traces: Vec::new(),
            };
        }

        let traces_with_senders = parse_all_msg_senders(trace_results.full_trace.trace);

        FullTransactionTraceWithLogs {
            tx_hash: trace_results.transaction_hash,
            tx_traces: traces_with_senders
                .into_iter()
                .enumerate()
                .map(|(idx, (trace, msg_sender))| TransactionTraceWithLogs {
                    trace,
                    logs: logs.clone(),
                    msg_sender,
                    trace_idx: idx as u64,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionTraceWithLogs {
    pub trace: TransactionTrace,
    pub logs: Vec<Log>,
    pub msg_sender: Address,
    pub trace_idx: u64,
}

impl TransactionTraceWithLogs {
    pub(crate) fn is_static_call(&self) -> bool {
        match &self.trace.action {
            Action::Call(call) => call.call_type == CallType::StaticCall,
            _ => false,
        }
    }

    fn get_from_addr(&self) -> Address {
        match &self.trace.action {
            Action::Call(call) => call.from,
            Action::Create(call) => call.from,
            Action::Reward(call) => call.author,
            Action::Selfdestruct(call) => call.address,
        }
    }

    fn get_to_address(&self) -> Address {
        match &self.trace.action {
            Action::Call(call) => call.to,
            Action::Create(_) => Address::default(),
            Action::Reward(_) => Address::default(),
            Action::Selfdestruct(call) => call.address,
        }
    }

    fn get_calldata(&self) -> Bytes {
        match &self.trace.action {
            Action::Call(call) => call.input.clone(),
            Action::Create(call) => call.init.clone(),
            _ => Bytes::default(),
        }
    }

    fn get_return_calldata(&self) -> Bytes {
        let Some(res) = &self.trace.result else {
            return Bytes::default();
        };
        match res {
            alloy_rpc_types_trace::parity::TraceOutput::Call(bytes) => bytes.output.clone(),
            _ => Bytes::default(),
        }
    }

    pub(crate) fn get_callframe_info(&self) -> CallFrameInfo<'_> {
        CallFrameInfo {
            trace_idx: self.trace_idx,
            call_data: self.get_calldata(),
            return_data: self.get_return_calldata(),
            target_address: self.get_to_address(),
            from_address: self.get_from_addr(),
            logs: &self.logs,
            delegate_logs: vec![],
            msg_sender: self.msg_sender,
            msg_value: self.get_msg_value(),
        }
    }

    fn get_msg_value(&self) -> U256 {
        match &self.trace.action {
            Action::Call(c) => c.value,
            Action::Create(c) => c.value,
            Action::Reward(r) => r.value,
            Action::Selfdestruct(_) => U256::ZERO,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]

pub struct DecodedCallData {
    pub function_name: String,
    pub call_data: Vec<DecodedParams>,
    pub return_data: Vec<DecodedParams>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedParams {
    pub field_name: String,
    pub field_type: String,
    pub value: String,
}

pub(crate) fn collect_delegated_traces<'a>(
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

fn parse_all_msg_senders(traces: Vec<TransactionTrace>) -> Vec<(TransactionTrace, Address)> {
    let mut traces_with_msg_senders: Vec<(TransactionTrace, Address)> = Vec::new();

    for current_trace in traces {
        let current_action = current_trace.action.clone();

        let msg_sender = if let Action::Call(c) = &current_action {
            if c.call_type == CallType::DelegateCall {
                if let Some((_, prev_msg_sender)) =
                    traces_with_msg_senders
                        .iter()
                        .rev()
                        .find(|(trace, _)| match &trace.action {
                            Action::Call(c) => c.call_type != CallType::DelegateCall,
                            Action::Create(_) => true,
                            _ => false,
                        })
                {
                    *prev_msg_sender
                } else {
                    panic!("should never be reached");
                }
            } else {
                match &current_action {
                    Action::Call(call) => call.from,
                    Action::Create(call) => call.from,
                    Action::Reward(call) => call.author,
                    Action::Selfdestruct(call) => call.address,
                }
            }
        } else {
            match &current_action {
                Action::Call(call) => call.from,
                Action::Create(call) => call.from,
                Action::Reward(call) => call.author,
                Action::Selfdestruct(call) => call.address,
            }
        };

        traces_with_msg_senders.push((current_trace, msg_sender));
    }

    traces_with_msg_senders
}
