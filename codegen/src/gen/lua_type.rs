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

use crate::parser::enums::EnumVariant;
use crate::parser::structs::StructField;
use crate::parser::Parser;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::Generics;

pub struct LuaType {
    name: Ident,
    generics: Generics,
}

impl LuaType {
    pub fn new(name: Ident, generics: Generics) -> Self {
        Self { name, generics }
    }
}

impl Parser for LuaType {
    type ParsedField = TokenStream;
    type ParsedVariant = Vec<TokenStream>;

    fn parse_field(&mut self, field: StructField) -> Self::ParsedField {
        let ty = field.ty;
        quote! {
            types.append(&mut <#ty as bp3d_lua::vm::util::LuaType>::lua_type());
        }
    }

    fn parse_variant(&mut self, field: EnumVariant) -> Self::ParsedVariant {
        let mut tokens = Vec::new();
        match field {
            EnumVariant::SingleField(v) => tokens.push(self.parse_field(v.field)),
            EnumVariant::MultiField(v) => {
                for v in v.fields {
                    tokens.push(self.parse_field(v));
                }
            }
            EnumVariant::None(_) => (),
        }
        tokens
    }

    fn gen_struct(self, parsed: Vec<Self::ParsedField>) -> TokenStream {
        let name = self.name;
        let generics = self.generics;
        quote! {
            impl #generics bp3d_lua::vm::util::LuaType for #name #generics {
                fn lua_type() -> Vec<bp3d_lua::vm::util::TypeName> {
                    let mut types = Vec::new();
                    #(#parsed)*
                    types
                }
            }
        }
    }

    fn gen_enum(self, parsed: Vec<Self::ParsedVariant>) -> TokenStream {
        let name = self.name;
        let generics = self.generics;
        let tokens = parsed.into_iter().map(|v| quote! { #(#v)* });
        quote! {
            impl #generics bp3d_lua::vm::util::LuaType for #name #generics {
                fn lua_type() -> Vec<bp3d_lua::vm::util::TypeName> {
                    let mut types = Vec::new();
                    #(#tokens)*
                    types
                }
            }
        }
    }
}
