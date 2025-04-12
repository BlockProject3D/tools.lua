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

use crate::decl_lib_func;
use crate::libs::Lib;
use crate::vm::function::types::RFunction;
use crate::vm::namespace::Namespace;
use crate::vm::table::Table;

decl_lib_func! {
    fn contains(src: &str, needle: &str) -> bool {
        src.contains(needle)
    }
}

decl_lib_func! {
    fn split<'a>(vm: &Vm, src: &str, pattern: &str) -> crate::vm::Result<Table<'a>> {
        let split = src.split(pattern);
        let mut tbl = Table::new(vm);
        for (i, v) in split.enumerate() {
            // Indices starts at 1 in lua.
            tbl.set((i + 1) as _, v)?;
        }
        Ok(tbl)
    }
}

decl_lib_func! {
    fn replace(src: &str, pattern: &str, replacement: &str) -> String {
        src.replace(pattern, replacement)
    }
}

decl_lib_func! {
    fn count(src: &str) -> u32 {
        src.chars().count() as u32
    }
}

decl_lib_func! {
    fn char_at(src: &str, pos: u32) -> Option<u32> {
        src.chars().nth(pos as usize).map(|v| v as u32)
    }
}

decl_lib_func! {
    fn from_string<'a>(src: &'a [u8]) -> Option<&'a str> {
        std::str::from_utf8(src).ok()
    }
}

decl_lib_func! {
    fn from_string_lossy(src: &[u8]) -> String {
        String::from_utf8_lossy(src).into()
    }
}

//TODO: implement function to substring respecting UTF8 codes (instead of the panicking rust version).

pub struct Utf8;

impl Lib for Utf8 {
    const NAMESPACE: &'static str = "bp3d.util.utf8";

    fn load(&self, namespace: &mut Namespace) -> crate::vm::Result<()> {
        namespace.add([
            ("contains", RFunction::wrap(contains)),
            ("split", RFunction::wrap(split)),
            ("replace", RFunction::wrap(replace)),
            ("count", RFunction::wrap(count)),
            ("charAt", RFunction::wrap(char_at)),
            ("fromString", RFunction::wrap(from_string)),
            ("fromStringLossy", RFunction::wrap(from_string_lossy))
        ])
    }
}
