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

//! Derive macros for the bee-event crate.

use quote::quote;
use syn::{parse_macro_input, DeriveInput, Lit, Meta};

#[proc_macro_derive(Event, attributes(name))]
pub fn derive_event(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    if input.attrs.len() != 1 {
        panic!("Invalid attributes number, derive_event requires one attribute.");
    }

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    let event_name = match input.attrs.get(0) {
        Some(name) => match name.parse_meta() {
            Ok(Meta::NameValue(meta)) => match &meta.lit {
                Lit::Str(lit) => lit.value(),
                _ => panic!("\"name\" attribute must be a string."),
            },
            _ => panic!("Expected argument `name = \"...\"`"),
        },
        None => unreachable!(),
    };

    // The generated implementation.
    let expanded = quote! {
        impl bee_event::Event for #name {
            fn name() -> &'static str {
                #event_name
            }

            fn interned_static() -> &'static std::thread::LocalKey<bee_event::EventNameCache> {
                thread_local! {
                    pub static INTERNED_NAME: bee_event::EventNameCache = Default::default();
                }
                &INTERNED_NAME
            }
        }
    };

    expanded.into()
}
