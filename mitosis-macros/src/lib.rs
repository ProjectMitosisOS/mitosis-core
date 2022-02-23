extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};

use quote::quote;

/// Create a getter to an extern variable in C (used for passing parameters)
/// 
/// ```
/// declare_module_param!(sample_long,u64);
/// ```
///

#[proc_macro]
pub fn declare_module_param(args: TokenStream) -> TokenStream {
    let arr: Vec<String> = args.to_string()
        .split(",").map(|x| String::from(x.trim())).collect();
    assert_eq!(arr.len(), 2);
    let param_name = Ident::new(&arr[0], Span::call_site());
    let type_name: proc_macro2::TokenStream = arr[1].parse().unwrap();
    quote! {
        pub struct #param_name;
        impl #param_name {
            pub fn read() -> #type_name {
                #[allow(improper_ctypes)]
                extern "C" {
                    static #param_name: #type_name;
                }
                unsafe { #param_name }
            }
        }
    }.into()
}
