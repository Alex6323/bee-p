// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// SecretDebug and SecretDisplay are two macros to derive `Debug` and `Display` for secret type.
/// Their actual implementation omits printint the actual secret.
/// Based on https://github.com/dtolnay/syn/blob/master/examples/heapsize/heapsize_derive/src/lib.rs

#[proc_macro_derive(SecretDebug)]
pub fn derive_secret_debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics std::fmt::Display for #name #ty_generics {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "<Omitted secret>")
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(SecretDisplay)]
pub fn derive_secret_display(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics std::fmt::Debug for #name #ty_generics {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "<Omitted secret>")
            }
        }
    };

    expanded.into()
}
