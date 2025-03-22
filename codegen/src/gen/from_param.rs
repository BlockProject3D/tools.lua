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
use crate::parser::enums::EnumVariant;
use crate::parser::Parser;
use crate::parser::structs::StructField;

pub struct FromParam {
    name: Ident,
    generics: Generics,
    is_index_based: bool,
    is_simpl_enum: bool
}

impl FromParam {
    pub fn new(name: Ident, generics: Generics) -> Self {
        Self { name, generics, is_index_based: false, is_simpl_enum: true }
    }
}

pub struct Field {
    name: Ident,
    from_param: TokenStream,
    try_from_param: TokenStream
}

impl Parser for FromParam {
    type ParsedField = Field;
    type ParsedVariant = Field;

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
        Field {
            name: name.clone(),
            from_param: quote! {
                #reader;
                top += 1;
                let #name: #ty = bp3d_lua::vm::function::FromParam::from_param(vm, top);
            },
            try_from_param: quote! {
                unsafe { #reader };
                top += 1;
                let #name: #ty = bp3d_lua::vm::function::FromParam::try_from_param(vm, top)?;
            }
        }
    }

    fn parse_variant(&mut self, variant: EnumVariant) -> Self::ParsedVariant {
        match variant {
            EnumVariant::SingleField(v) => {
                self.is_simpl_enum = false;
                let ty = v.field.ty;
                let name = self.name.clone();
                let variant = v.unique_name;
                Field {
                    name: variant.clone(),
                    from_param: quote! {
                        match <#ty as bp3d_lua::vm::function::FromParam>::try_from_param(vm, index) {
                            Some(v) => return #name::#variant(v),
                            None => ()
                        };
                    },
                    try_from_param: quote! {
                        match <#ty as bp3d_lua::vm::function::FromParam>::try_from_param(vm, index) {
                            Some(v) => return Some(#name::#variant(v)),
                            None => ()
                        };
                    }
                }
            }
            EnumVariant::MultiField(_) => {
                panic!("Multi-field enum variants are not supported");
            }
            EnumVariant::None(variant) => {
                let name = self.name.clone();
                let vname = variant.to_string();
                Field {
                    name: variant.clone(),
                    from_param: quote! {
                        match enum_name == #vname {
                            true => return #name::#variant,
                            false => ()
                        };
                    },
                    try_from_param: quote! {
                        match enum_name == #vname {
                            true => return Some(#name::#variant),
                            false => ()
                        };
                    }
                }
            }
        }
    }

    fn gen_struct(self, parsed: Vec<Self::ParsedField>) -> TokenStream {
        let name = self.name;
        let generics = self.generics;
        let lifetime = generics.lifetimes().next().map(|v| v.into_token_stream()).unwrap_or(quote! { '_ });
        let from_params = parsed.iter().map(|field| &field.from_param);
        let try_from_params = parsed.iter().map(|field| &field.try_from_param);
        let values = parsed.iter().map(|field| &field.name);
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
                    unsafe { bp3d_lua::ffi::laux::luaL_checktype(vm.as_ptr(), index, bp3d_lua::ffi::lua::Type::Table) };
                    let mut top = vm.top();
                    #(#from_params)*
                    #end
                }

                fn try_from_param(vm: &#lifetime bp3d_lua::vm::Vm, index: i32) -> Option<Self> {
                    bp3d_lua::vm::value::util::ensure_type_equals(vm, index, bp3d_lua::ffi::lua::Type::Table).ok()?;
                    let mut top = vm.top();
                    let mut f = || -> Option<Self> {
                        #(#try_from_params)*
                        Some(#end)
                    };
                    match f() {
                        Some(v) => Some(v),
                        None => {
                            // Reset stack position.
                            unsafe { bp3d_lua::ffi::lua::lua_settop(vm.as_ptr(), top) };
                            None
                        }
                    }
                }
            }

            unsafe impl #generics bp3d_lua::util::SimpleDrop for #name #generics { }
        }
    }

    fn gen_enum(self, parsed: Vec<Self::ParsedVariant>) -> TokenStream {
        let name = self.name;
        let generics = self.generics;
        let lifetime = generics.lifetimes().next().map(|v| v.into_token_stream()).unwrap_or(quote! { '_ });
        let from_params = parsed.iter().map(|field| &field.from_param);
        let try_from_params = parsed.iter().map(|field| &field.try_from_param);
        quote! {
            impl #generics bp3d_lua::vm::function::FromParam<#lifetime> for #name #generics {
                unsafe fn from_param(vm: &#lifetime bp3d_lua::vm::Vm, index: i32) -> Self {
                    #(#from_params)*
                    bp3d_lua::ffi::laux::luaL_error(vm.as_ptr(), "Unable to find a type satisfying constraints");
                    std::hint::unreachable_unchecked()
                }

                fn try_from_param(vm: &#lifetime bp3d_lua::vm::Vm, index: i32) -> Option<Self> {
                    #(#try_from_params)*
                    None
                }
            }

            unsafe impl #generics bp3d_lua::util::SimpleDrop for #name #generics { }
        }
    }
}
