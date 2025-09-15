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

use crate::gen::IntoLua;
use crate::parser::enums::EnumVariant;
use crate::parser::structs::StructField;
use crate::parser::Parser;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::Generics;

pub struct IntoParam(IntoLua);

impl IntoParam {
    pub fn new(name: Ident, generics: Generics) -> Self {
        Self(IntoLua::new(name, generics))
    }
}

impl Parser for IntoParam {
    type ParsedField = TokenStream;
    type ParsedVariant = TokenStream;

    fn parse_field(&mut self, field: StructField) -> Self::ParsedField {
        self.0.parse_field(field)
    }

    fn parse_variant(&mut self, variant: EnumVariant) -> Self::ParsedVariant {
        match variant {
            EnumVariant::SingleField(v) => {
                let name = v.unique_name;
                let ty = v.field.ty;
                quote! {
                    Self::#name(v) => <#ty as bp3d_lua::vm::function::IntoParam>::into_param(v, vm),
                }
            }
            EnumVariant::MultiField(_) => panic!("Multi-field enum variants are not supported"),
            EnumVariant::None(name) => {
                let str = name.to_string();
                quote! {
                    Self::#name => <&str as bp3d_lua::vm::function::IntoParam>::into_param(#str, vm),
                }
            }
        }
    }

    fn gen_struct(self, parsed: Vec<Self::ParsedField>) -> TokenStream {
        let name = self.0.name;
        let generics = self.0.generics;
        quote! {
            unsafe impl #generics bp3d_lua::vm::function::IntoParam for #name #generics {
                fn into_param(self, vm: &bp3d_lua::vm::Vm) -> i32 {
                    let mut tbl = bp3d_lua::vm::table::Table::new(vm);
                    #(#parsed)*
                    1
                }
            }
        }
    }

    fn gen_enum(self, parsed: Vec<Self::ParsedVariant>) -> TokenStream {
        let name = self.0.name;
        let generics = self.0.generics;
        quote! {
            unsafe impl #generics bp3d_lua::vm::function::IntoParam for #name #generics {
                fn into_param(self, vm: &bp3d_lua::vm::Vm) -> i32 {
                    match self {
                        #(#parsed)*
                    }
                }
            }
        }
    }
}
