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

mod parser;
mod gen;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use crate::parser::Parser;
use crate::gen::{FromParam, IntoParam, LuaType};

#[proc_macro_derive(FromParam)]
pub fn from_param(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, generics, .. } = parse_macro_input!(input);
    FromParam::new(ident, generics).parse(data).into()
}

#[proc_macro_derive(IntoParam)]
pub fn into_param(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, generics, .. } = parse_macro_input!(input);
    IntoParam::new(ident, generics).parse(data).into()
}

#[proc_macro_derive(LuaType)]
pub fn lua_type(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, generics, .. } = parse_macro_input!(input);
    LuaType::new(ident, generics).parse(data).into()
}
