extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};

use quote::quote;

/// Create a getter to an extern variable in C (used for passing parameters)
/// 
/// ```
/// declare_module!(sample_long,u64);
/// ```
///

#[proc_macro]
pub fn declare_module(args: TokenStream) -> TokenStream {
    let arr: Vec<String> = args.to_string()
        .split(",").map(|x| String::from(x.trim())).collect();
    assert_eq!(arr.len(), 2);
    let func_name = format!("get_{}", arr[0]);
    let func_name = Ident::new(&func_name, Span::call_site());
    let param_name = Ident::new(&arr[0], Span::call_site());
    let type_name = Ident::new(&arr[1], Span::call_site());
    quote! {
        pub fn #func_name() -> #type_name {
            #[allow(improper_ctypes)]
            extern "C" {
                static #param_name: #type_name;
            }
            unsafe { #param_name }
        }
    }.into()
}
