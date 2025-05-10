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

mod gen;
mod parser;

use crate::gen::{FromParam, IntoParam, LuaType};
use crate::parser::Parser;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(FromParam)]
pub fn from_param(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = parse_macro_input!(input);
    FromParam::new(ident, generics).parse(data).into()
}

#[proc_macro_derive(IntoParam)]
pub fn into_param(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = parse_macro_input!(input);
    IntoParam::new(ident, generics).parse(data).into()
}

#[proc_macro_derive(LuaType)]
pub fn lua_type(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = parse_macro_input!(input);
    LuaType::new(ident, generics).parse(data).into()
}

#[proc_macro]
pub fn decl_lua_plugin(input: TokenStream) -> TokenStream {
    let ident = parse_macro_input!(input as Ident);
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let func_name = format!("bp3d_lua_{}_register_{}", crate_name.to_uppercase(), ident.to_string());
    let func = Ident::new(&func_name, ident.span());
    let q = quote! {
        #[no_mangle]
        extern "Rust" fn #func(l: bp3d_lua::ffi::lua::State) -> bp3d_lua::vm::Result<()> {
            let vm = unsafe { bp3d_lua::vm::Vm::from_raw(l) };
            #ident.register(&vm)
        }
    };
    q.into()
}

#[proc_macro]
pub fn decl_lua_lib(input: TokenStream) -> TokenStream {
    let ident = parse_macro_input!(input as Ident);
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let rustc_const_name = format!("BP3D_LUA_{}_RUSTC_VERSION", crate_name.to_uppercase());
    let bp3d_lua_const_name = format!("BP3D_LUA_{}_ENGINE_VERSION", crate_name.to_uppercase());
    let const_name = format!("BP3D_LUA_{}_VERSION", crate_name.to_uppercase());
    let rustc_const = Ident::new(&rustc_const_name, ident.span());
    let bp3d_lua_const = Ident::new(&bp3d_lua_const_name, ident.span());
    let cons = Ident::new(&const_name, ident.span());
    let q = quote! {
        #[no_mangle]
        extern "C" const #rustc_const: *const std::ffi::c_char = bp3d_lua::module::RUSTC_VERSION.as_ptr() as _;

        #[no_mangle]
        extern "C" const #bp3d_lua_const: *const std::ffi::c_char = bp3d_lua::module::VERSION.as_ptr() as _;

        #[no_mangle]
        extern "C" const #cons: *const std::ffi::c_char = concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as _;
    };
    q.into()
}
