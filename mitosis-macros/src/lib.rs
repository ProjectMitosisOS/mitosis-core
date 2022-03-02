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


/// Generate code for declaring global variables in rust
/// Note: can only be used in lib.rs
///
/// ```
/// declare_global!(TEST,u64);
///
/// ```
///
/// The global variable can be init via:
/// ```
/// TEST::init(some exp);
/// ```
///
/// The mut reference to global variable can be accessed via:
/// ```
/// let test = unsafe { TEST::get_mut() };
/// ```
///
#[proc_macro]
pub fn declare_global(args: TokenStream) -> TokenStream {
    let arr: Vec<String> = args.to_string()
        .split(",").map(|x| String::from(x.trim())).collect();
    assert_eq!(arr.len(), 2);
    let param_name = Ident::new(&arr[0], Span::call_site());

    let param_type: proc_macro2::TokenStream = arr[1].parse().unwrap();

    quote! {
        static mut #param_name : Option<#param_type> = None;

        mod #param_name {
            pub unsafe fn init(v : #param_type) {
                crate::#param_name = Some(v);
            }

            #[inline]
            pub unsafe fn get_mut() -> &'static mut #param_type {
                match crate::#param_name {
                    Some(ref mut x) => &mut *x,
                    None => panic!()
                }
            }

            #[inline]
            pub unsafe fn get_ref() -> &'static #param_type {
                match crate::#param_name {
                    Some(ref x) => & *x,
                    None => panic!()
                }
            }

            #[inline]
            pub unsafe fn drop() {
                crate::#param_name = None;
            }
        }
    }.into()
}
