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

use proc_macro2::TokenStream;
use syn::Data;
use crate::parser::structs::{StructField, StructParser};

pub trait Parser: Sized {
    type ParsedField;
    type ParsedVariant;

    fn parse_field(&mut self, field: StructField) -> Self::ParsedField;

    fn gen_struct(self, parsed: Vec<Self::ParsedField>) -> TokenStream;

    fn parse(mut self, data: Data) -> TokenStream {
        match data {
            Data::Struct(v) => {
                
                let mut parser = StructParser::new();
                let mut parsed = Vec::new();
                for v in v.fields {
                    parsed.push(self.parse_field(parser.parse(v)));
                }
                self.gen_struct(parsed)
            }
            Data::Enum(_) => {
                /*let mut parser = Self::new_enum_parser(params);
                for v in v.variants {
                    parser.parse_variant(v);
                }*/
                todo!()
            }
            _ => panic!("Unsupported type")
        }
    }
}
