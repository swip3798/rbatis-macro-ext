use proc_macro::TokenStream;
use syn::{self, parse_macro_input, AttributeArgs, ItemFn};

mod macros;
pub(crate) mod util;

#[cfg(feature="question-marker")]
pub(crate) fn symbol(_number: u32) -> String {
    "?".to_string()
}

#[cfg(feature="dollar-marker")]
pub(crate) fn symbol(number: u32) -> String {
    format!("${number}")
}

#[cfg(all(not(feature="dollar-marker"), not(feature="question-marker")))]
pub(crate) fn symbol(_number: usize) -> String {
    "?".to_string()
}

#[proc_macro_attribute]
pub fn ext_sql(args: TokenStream, func: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let target_fn: ItemFn = syn::parse(func).unwrap();
    let stream = macros::impl_macro_sql(&target_fn, &args);
    stream
}