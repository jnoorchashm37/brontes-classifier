use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, Index, Token, parenthesized, parse::Parse};

use super::ACTION_SIG_NAME;

#[derive(Debug)]
pub struct ActionDispatch {
    // required for all
    struct_name: Ident,
    protocol_enum: Ident,
    output_type: Ident,
    rest: Vec<Ident>,
}

impl ActionDispatch {
    pub fn expand(self) -> syn::Result<TokenStream> {
        let Self {
            struct_name,
            protocol_enum,
            output_type,
            rest,
        } = self;

        if rest.is_empty() {
            // Generate a compile_error! invocation as part of the output TokenStream
            return Err(syn::Error::new(
                Span::call_site(),
                "need classifiers to dispatch to",
            ));
        }
        let (var_name, const_fns): (Vec<_>, Vec<_>) = rest
            .iter()
            .enumerate()
            .map(|(i, ident)| {
                (
                    Ident::new(&format!("VAR_{i}"), ident.span()),
                    Ident::new(&format!("{ACTION_SIG_NAME}_{}", ident), ident.span()),
                )
            })
            .unzip();

        let (i, name): (Vec<Index>, Vec<&Ident>) = rest
            .iter()
            .enumerate()
            .map(|(i, n)| (Index::from(i), n))
            .unzip();

        let match_stmt = expand_match_dispatch(&rest, &var_name, i);

        let o = quote!(

            impl #protocol_enum {
                pub const fn to_byte(&self) -> u8 {
                    *self as u8
                }
            }

            #[derive(Default, Debug)]
            pub struct #struct_name(#(pub #name,)*);

            impl ::brontes_classifier::action::ActionCollection for #struct_name {
                type DispatchOut = #output_type;
                type ProtocolContext = #protocol_enum;

                fn dispatch<DB: brontes_classifier::context::DataContext<#protocol_enum>>(
                    &self,
                    call_info: ::brontes_classifier::types::CallFrameInfo<'_>,
                    data_ctx: &DB,
                    block: u64,
                    tx_idx: u64,
                ) -> Option<#output_type> {


                    let protocol_fetched: #protocol_enum =
                        ::brontes_classifier::context::DataContext::get_protocol(data_ctx, call_info.target_address).ok()?;
                    let protocol_byte = protocol_fetched.to_byte();

                    if call_info.call_data.len() < 4 {
                        return None
                    }

                    let hex_selector = ::alloy_primitives::Bytes::copy_from_slice(
                        &call_info.call_data[0..4]);

                    let sig = ::alloy_primitives::FixedBytes::<4>::from_slice(
                        &call_info.call_data[0..4]).0;

                    let mut sig_w_byte= [0u8; 5];
                    sig_w_byte[0..4].copy_from_slice(&sig);
                    sig_w_byte[4] = protocol_byte;


                    #(
                        const #var_name: [u8; 5] = #const_fns();
                    )*;

                    #match_stmt

                }
            }
        );

        Ok(o)
    }
}

impl Parse for ActionDispatch {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let paren_input;
        parenthesized!(paren_input in input);

        let struct_name: Ident = paren_input.parse()?;
        paren_input.parse::<Token![,]>()?;
        let protocol_enum: Ident = paren_input.parse()?;

        input.parse::<Token![=]>()?;
        input.parse::<Token![>]>()?;

        let output_type: Ident = input.parse()?;
        input.parse::<Token![|]>()?;

        let mut rest = vec![input.parse::<Ident>()?];
        while input.parse::<Token![,]>().is_ok() {
            rest.push(input.parse::<Ident>()?);
        }

        if !input.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "Unwanted input at end of macro",
            ));
        }

        Ok(Self {
            rest,
            protocol_enum,
            output_type,
            struct_name,
        })
    }
}

fn expand_match_dispatch(
    reg_name: &[Ident],
    var_name: &[Ident],
    var_idx: Vec<Index>,
) -> TokenStream {
    quote!(
        match sig_w_byte {
        #(
            #var_name => {
                let target_address = call_info.target_address;
                ::brontes_classifier::action::IntoAction::decode_call_trace(
                        &self.#var_idx,
                        call_info,
                        block,
                        tx_idx,
                        data_ctx
                    ).inspect_err(|e| {
                        ::tracing::warn!(error=%e,
                            "classifier: {} failed on function sig: {:?} for address: {:?}",
                            stringify!(#reg_name),
                            hex_selector,
                            target_address.0,
                        );

                    }).ok()
            }
            )*

            _ => {
            let target_address = call_info.target_address;
            ::tracing::debug!(
                "no inspector for function selector: {:?} with contract address: {:?}",
                hex_selector,
                target_address.0,
            );

                None
            }
        }
    )
}
