use quote::{ToTokens, quote};
use syn::ExprClosure;

pub struct ClosureDispatch {
    logs: bool,
    call_data: bool,
    return_data: bool,
    closure: ExprClosure,
}

impl ClosureDispatch {
    pub fn new(logs: bool, call_data: bool, return_data: bool, closure: ExprClosure) -> Self {
        Self {
            closure,
            call_data,
            return_data,
            logs,
        }
    }
}

impl ToTokens for ClosureDispatch {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let closure = &self.closure;

        let call_data = self
            .call_data
            .then_some(quote!(call_data,))
            .unwrap_or_default();

        let return_data = self
            .return_data
            .then_some(quote!(return_data,))
            .unwrap_or_default();

        let log_data = self.logs.then_some(quote!(log_data,)).unwrap_or_default();

        tokens.extend(quote!(
            let fixed_fields = call_info.get_fixed_fields();
            (#closure)
            (
                fixed_fields,
                #call_data
                #return_data
                #log_data
                db_ctx
            )
        ))
    }
}
