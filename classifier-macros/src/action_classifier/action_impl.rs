use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Error, ExprClosure, Ident, LitBool, Path, Token, bracketed, parenthesized,
    parse::Parse,
    spanned::Spanned,
    token::{Paren, Star},
};

use super::{ACTION_SIG_NAME, data_preparation::CallDataParsing, logs::LogConfig};

pub struct ActionMacro {
    output_type: Ident,
    protocol_enum: Ident,
    // required for all
    protocol_path: Path,
    path_to_call: Path,
    action_type: Ident,
    exchange_name_w_call: Ident,
    log_types: Vec<LogConfig>,
    /// whether we want logs or not
    give_logs: bool,
    /// whether we want return data or not
    give_returns: bool,
    /// whether we want call_data or not
    give_call_data: bool,
    // whether we pass down logs from delegate call in the same call frame
    include_delegated_logs: bool,
    /// The closure that we use to construct the normalized type
    call_function: ExprClosure,
}

impl ActionMacro {
    pub fn expand(self) -> syn::Result<TokenStream> {
        let Self {
            output_type,
            protocol_enum,
            exchange_name_w_call,
            protocol_path,
            action_type,
            path_to_call,
            log_types,
            give_logs,
            give_call_data,
            include_delegated_logs,
            give_returns,
            call_function,
        } = self;

        let call_data = CallDataParsing::new(
            give_logs,
            give_call_data,
            give_returns,
            include_delegated_logs,
            &exchange_name_w_call,
            &action_type,
            &path_to_call,
            &log_types,
            call_function,
        );

        let call_fn_name = Ident::new(
            &format!("{ACTION_SIG_NAME}_{}", exchange_name_w_call),
            Span::call_site(),
        );

        let mut return_import = path_to_call.clone();
        let mut call = return_import.segments.pop().ok_or(syn::Error::new(
            return_import.span(),
            "invalid call import type",
        ))?;
        let call_ident = call.value().ident.to_string();
        let solidity = call_ident[0..call_ident.len() - 4].to_string() + "Return";

        call.value_mut().ident = Ident::new(&solidity, call.span());
        return_import.segments.push(call.into_value());

        let combined_output = quote! { #output_type::#action_type(result) };

        Ok(quote!(
            #[allow(unused_imports)]
            use #path_to_call;
            #[allow(unused_imports)]
            use #return_import;

            #[allow(non_snake_case)]
            pub const fn #call_fn_name() -> [u8; 5] {
                ::alloy_primitives::FixedBytes::new(
                        <#path_to_call as ::alloy_sol_types::SolCall>::SELECTOR
                    )
                    .concat_const(
                    ::alloy_primitives::FixedBytes::new(
                        [#protocol_path.to_byte()]
                        )
                    ).0
            }

            #[derive(Debug, Default)]
            pub struct #exchange_name_w_call;

            impl ::brontes_classifier::action::IntoAction for #exchange_name_w_call {
                type DecodeOut = #output_type;
                type ProtocolContext = #protocol_enum;

                fn decode_call_trace<DB: ::brontes_classifier::context::DataContext<#protocol_enum>>(
                    &self,
                    call_info: ::brontes_classifier::types::CallFrameInfo<'_>,
                    block: u64,
                    tx_idx: u64,
                    db_ctx: &DB
                    ) -> ::eyre::Result<#output_type> {
                    #call_data.map(|result| #combined_output)

                }
            }
        ))
    }
}

impl Parse for ActionMacro {
    fn parse(mut input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let protocol_enum = content.parse()?;
        content.parse::<Token![,]>()?;
        let output_type = content.parse()?;

        input.parse::<Token![,]>()?;
        let protocol_path = parse_protocol_path(&mut input)?;
        input.parse::<Token![,]>()?;

        let path_to_call = parse_decode_fn_path(&mut input)?;
        input.parse::<Token![,]>()?;

        let action_type: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        let possible_logs = parse_logs(&mut input)?;
        input.parse::<Token![,]>()?;

        let (logs, return_data, call_data, include_delegated_logs) = parse_config(&mut input)?;
        let call_function = parse_closure(&mut input)?;

        let uppercase_path_to_call = uppercase_first_char(
            &path_to_call.segments[path_to_call.segments.len() - 1]
                .ident
                .to_string(),
        );

        let exchange_name_w_call = Ident::new(
            &format!(
                "{}{}",
                protocol_path.segments[protocol_path.segments.len() - 1].ident,
                uppercase_path_to_call
            ),
            Span::call_site(),
        );

        Ok(Self {
            output_type,
            protocol_enum,
            path_to_call,
            give_returns: return_data,
            log_types: possible_logs,
            call_function,
            give_logs: logs,
            give_call_data: call_data,
            include_delegated_logs,
            action_type,
            protocol_path,
            exchange_name_w_call,
        })
    }
}

fn parse_closure(input: &mut syn::parse::ParseStream) -> syn::Result<ExprClosure> {
    let call_function: ExprClosure = input.parse()?;
    if call_function.asyncness.is_some() {
        return Err(syn::Error::new(input.span(), "closure cannot be async"));
    }

    if !input.is_empty() {
        return Err(syn::Error::new(
            input.span(),
            "There should be no values after the call function",
        ));
    }

    if call_function.asyncness.is_some() {
        return Err(syn::Error::new(input.span(), "closure cannot be async"));
    }

    if !input.is_empty() {
        return Err(syn::Error::new(
            input.span(),
            "There should be no values after the call function",
        ));
    }

    Ok(call_function)
}

fn parse_config(input: &mut syn::parse::ParseStream) -> syn::Result<(bool, bool, bool, bool)> {
    let mut logs = false;
    let mut return_data = false;
    let mut call_data = false;
    let mut include_delegated_logs = false;

    while !input.peek(Token![|]) {
        let arg: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let enabled: LitBool = input.parse()?;

        match arg.to_string().to_lowercase().as_str() {
            "logs" => logs = enabled.value(),
            "call_data" => call_data = enabled.value(),
            "return_data" => return_data = enabled.value(),
            "include_delegated_logs" => include_delegated_logs = enabled.value(),
            _ => {
                return Err(Error::new(
                    arg.span(),
                    format!(
                        "{} is not a valid config option, valid options are: \n logs , call_data, \
                         return_data , include_delegated_logs",
                        arg,
                    ),
                ));
            }
        }
        input.parse::<Token![,]>()?;
    }

    Ok((logs, return_data, call_data, include_delegated_logs))
}

pub fn parse_protocol_path(input: &mut syn::parse::ParseStream) -> syn::Result<Path> {
    let protocol_path: Path = input.parse().map_err(|_| {
        syn::Error::new(
            input.span(),
            "No Protocol Found, Should be Protocol::<ProtocolVarient>",
        )
    })?;

    if protocol_path.segments.len() < 2 {
        return Err(syn::Error::new(
            protocol_path.span(),
            "incorrect path, Should be Protocol::<ProtocolVarient>",
        ));
    }

    // let should_protocol = &protocol_path.segments[protocol_path.segments.len() - 2].ident;
    // if !should_protocol.to_string().starts_with("Protocol") {
    //     return Err(syn::Error::new(
    //         should_protocol.span(),
    //         "incorrect path, Should be Protocol::<ProtocolVarient>",
    //     ));
    // }
    Ok(protocol_path)
}

fn parse_decode_fn_path(input: &mut syn::parse::ParseStream) -> syn::Result<Path> {
    let fn_path: Path = input.parse().map_err(|_| {
        syn::Error::new(
            input.span(),
            "No path to alloy fn found, Should be path::to::alloy::call::fn_nameCall",
        )
    })?;

    if fn_path.segments.len() < 2 {
        return Err(syn::Error::new(
            fn_path.span(),
            "incorrect path, Should be <crate>::<path_to>::ProtocolModName::FnCall",
        ));
    }

    Ok(fn_path)
}

fn parse_logs(input: &mut syn::parse::ParseStream) -> syn::Result<Vec<LogConfig>> {
    let mut log_types = Vec::new();
    let content;
    bracketed!(content in input);

    loop {
        let mut can_repeat = false;
        let mut ignore_before = false;

        if content.peek(Token![..]) {
            let _ = content.parse::<Token![..]>()?;

            ignore_before = true;
        }

        let fallbacks;
        // have fallback
        let buf = if content.peek(Paren) {
            parenthesized!(fallbacks in content);
            &fallbacks
        } else {
            &content
        };

        let Ok(log_type) = buf.parse::<Ident>() else {
            break;
        };

        let mut fallback = Vec::new();

        while buf.peek(Token![|]) {
            let _ = buf.parse::<Token![|]>()?;
            let Ok(log_type) = buf.parse::<Ident>() else {
                break;
            };
            fallback.push(log_type);
        }

        if content.peek(Star) {
            let _ = content.parse::<Star>()?;
            can_repeat = true;
        }

        log_types.push(LogConfig {
            ignore_before,
            can_repeat,
            log_ident: log_type,
            log_fallbacks: fallback,
        });

        let Ok(_) = content.parse::<Token![,]>() else {
            break;
        };
    }

    Ok(log_types)
}

fn uppercase_first_char(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
