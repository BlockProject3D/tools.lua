// Copyright (c) 2025, BlockProject 3D
//
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above copyright notice,
//       this list of conditions and the following disclaimer in the documentation
//       and/or other materials provided with the distribution.
//     * Neither the name of BlockProject 3D nor the names of its contributors
//       may be used to endorse or promote products derived from this software
//       without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::Generics;
use crate::parser::Parser;
use crate::parser::structs::StructField;

pub struct FromParam {
    name: Ident,
    generics: Generics,
    is_index_based: bool
}

impl FromParam {
    pub fn new(name: Ident, generics: Generics) -> Self {
        Self { name, generics, is_index_based: false }
    }
}

impl Parser for FromParam {
    type ParsedField = (Ident, TokenStream);
    type ParsedVariant = ();

    fn parse_field(&mut self, field: StructField) -> Self::ParsedField {
        let name = field.unique_name;
        let ty = field.ty;
        self.is_index_based = field.unique_name_is_index;
        let reader = if field.unique_name_is_index {
            let index = field.index + 1; //Lua indices starts at 1, not 0.
            quote! { bp3d_lua::ffi::lua::lua_rawgeti(vm.as_ptr(), index, #index as _) }
        } else {
            quote! { bp3d_lua::ffi::lua::lua_getfield(vm.as_ptr(), index, bp3d_lua::c_stringify!(#name).as_ptr()) }
        };
        (name.clone(), quote! {
            #reader;
            top += 1;
            let #name: #ty = bp3d_lua::vm::function::FromParam::from_param(&vm, top);
        })
    }

    fn gen_struct(self, parsed: Vec<Self::ParsedField>) -> TokenStream {
        let name = self.name;
        let generics = self.generics;
        let lifetime = generics.lifetimes().next().map(|v| v.into_token_stream()).unwrap_or(quote! { '_ });
        let from_params = parsed.iter().map(|(_, v)| v);
        let values = parsed.iter().map(|(k, _)| k);
        let end = if self.is_index_based {
            quote! {
                #name(#(#values),*)
            }
        } else {
            quote! {
                #name { #(#values),* }
            }
        };
        quote! {
            impl #generics bp3d_lua::vm::function::FromParam<#lifetime> for #name #generics {
                unsafe fn from_param(vm: &#lifetime bp3d_lua::vm::Vm, index: i32) -> Self {
                    let mut top = vm.top();
                    #(#from_params)*
                    #end
                }
            }
            
            unsafe impl #generics bp3d_lua::util::SimpleDrop for #name #generics { }
        }
    }
}
