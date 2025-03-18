mod action_classifier;

use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::action_classifier::{ActionDispatch, ActionMacro};

#[proc_macro]
/// the action impl macro deals with automatically parsing the data needed for
/// underlying actions. The use is as followed
/// ```ignore
/// action_impl!(ProtocolPath, PathToCall, CallType, [LogType / 's], [logs: bool , call_data: bool, return_data: bool])
/// ```
/// The generated structs name will be as the following:
///  &lt;LastIdentInProtocolPath&gt; + &lt;LastIdentInPathToCall&gt;
/// Example:
/// a macro invoked with
///     Protocol::UniswapV2,
///     crate::UniswapV2::swapCall,
///
/// becomes: UniswapV2swapCall.
/// This is done to avoid naming conflicts between classifiers as this is name
/// will always be unique.
///
/// The Array of log types are expected to be in the order that they are emitted
/// in. Otherwise the decoding will fail
///
///  ## Examples
/// ```ignore
/// action_impl!(
///     Protocol::UniswapV2,
///     crate::UniswapV2::swapCall,
///     Swap,
///     [..Swap],
///     logs: true,
///     |index,
///     from_address: Address,
///     target_address: Address,
///     msg_sender: Address,
///     log_data: UniswapV2swapCallLogs| { <body> });
///
/// action_impl!(
///     Protocol::UniswapV2,
///     crate::UniswapV2::mintCall,
///     Mint,
///     [..Mint],
///     logs: true,
///     call_data: true,
///     |index,
///      from_address: Address,
///      target_address: Address,
///      msg_sender: Address,
///      call_data: mintCall,
///      log_data: UniswapV2mintCallLogs|  { <body> });
/// ```
///
/// # Logs Config
/// NOTE: all log modifiers are compatible with each_other
/// ## Log Ignore Before
/// if you want to ignore all logs that occurred before a certain log,
/// prefix the log with .. ex `..Mint`.
///
/// ## Log Repeating
/// if a log is repeating and dynamic in length, use `*` after the log
/// to mark that there is a arbitrary amount of these logs emitted.
/// ex `Transfer*` or `..Transfer*`
///
/// ## Fallback logs.
/// in the case that you might need a fallback log, these can be defined by
/// wrapping the names in parens. e.g (Transfer | SpecialTransfer).
/// this will try to decode transfer first and if it fails, special transfer.
/// Fallback logs are configurable with other log parsing options. this means
/// you can do something like ..(Transfer | SpecialTransfer) or ..(Transfer |
/// SpecialTransfer)*
///
///
/// the fields `call_data`, `return_data` and `log_data` are only put into the
/// closure if specified they are always in this order, for example if you put
///  
///  ```return_data: true```
///  then then the closure would be as followed
///  ```|index, from_address, target_address, return_data|```
///
/// for
///  ```ignore
///  log_data: true,
///  call_data: true
///  ````
///  ```|index, from_address, target_address, return_data, log_data|```
pub fn action_impl(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as ActionMacro)
        .expand()
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
/// action_dispatch macro crates a struct that automatically dispatches
/// the given trace information to the proper action classifier. its invoked as
/// the following:
/// ```ignore
/// action_dispatch!(<DispatchStructName>, [action_classifier_names..],);
/// ```
/// an actual example would be
/// ```ignore
/// # use brontes_macros::{action_dispatch, action_impl};
/// # use brontes_pricing::Protocol;
/// # use brontes_types::normalized_actions::NormalizedSwap;
/// # use alloy_primitives::Address;
/// # use brontes_database::libmdbx::tx::CompressedLibmdbxTx;
///
/// action_impl!(
///     Protocol::UniswapV2,
///     crate::UniswapV2::swapCall,
///     Swap,
///     [Ignore<Sync>, Swap],
///     call_data: true,
///     logs: true,
///     |trace_index,
///     from_address: Address,
///     target_address: Address,
///      msg_sender: Address,
///     call_data: swapCall,
///     log_data: UniswapV2swapCallLogs,
///     db_tx: &DB| {
///         todo!()
///     }
/// );
///
/// action_dispatch!(ClassifierDispatch, UniswapV2swapCall);
/// ```
pub fn action_dispatch(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as ActionDispatch)
        .expand()
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
