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

use bp3d_util::simple_error;
use crate::vm::table::Table as LuaTable;
use crate::decl_lib_func;
use crate::ffi::lua::Type;
use crate::libs::Lib;
use crate::vm::error::{Error, TypeError};
use crate::vm::function::types::RFunction;
use crate::vm::namespace::Namespace;
use crate::vm::value::any::AnyValue;

decl_lib_func! {
    fn update(dst: LuaTable, src: LuaTable) -> crate::vm::Result<()> {
        let mut src = src;
        let mut dst = dst;
        for res in src.iter() {
            let (k, v) = res?;
            match k {
                AnyValue::String(name) => {
                    dst.set_field(name, v)?
                },
                AnyValue::Number(num) => dst.set(num as _, v)?,
                _ => return Err(Error::Type(TypeError {
                    expected: Type::String,
                    actual: k.ty()
                }))
            }
        }
        Ok(())
    }
}

decl_lib_func! {
    fn count(src: LuaTable) -> u64 {
        src.len() as _
    }
}

fn to_string_rec(prefix: String, mut table: LuaTable) -> crate::vm::Result<Vec<String>> {
    let mut lines = Vec::new();
    for res in table.iter() {
        let (k, v) = res?;
        match v {
            AnyValue::Table(v) => {
                lines.push(format!("{}:", k));
                lines.extend(to_string_rec(prefix.clone() + "    ", v)?);
            },
            v => lines.push(format!("{}: {}", k, v))
        }
    }
    Ok(lines)
}

decl_lib_func! {
    fn to_string(src: LuaTable) -> crate::vm::Result<String> {
        to_string_rec("".into(), src).map(|v| v.join("\n"))
    }
}

decl_lib_func! {
    fn contains(src: LuaTable, value: AnyValue) -> crate::vm::Result<bool> {
        let mut src = src;
        for res in src.iter() {
            let (_, v) = res?;
            if v == value {
                return Ok(true)
            }
        }
        Ok(false)
    }
}

decl_lib_func! {
    fn contains_key(src: LuaTable, key: AnyValue) -> crate::vm::Result<bool> {
        let mut src = src;
        for res in src.iter() {
            let (k, _) = res?;
            if k == key {
                return Ok(true)
            }
        }
        Ok(false)
    }
}

simple_error! {
    ProtectError {
        NewIndex => "attempt to set value into protected table."
    }
}

decl_lib_func! {
    fn __newindex() -> Result<(), ProtectError> {
        Err(ProtectError::NewIndex)
    }
}

decl_lib_func! {
    fn protect<'a>(vm: &Vm, src: LuaTable) -> crate::vm::Result<LuaTable<'a>> {
        let mut wrapper = LuaTable::new(vm);
        let mut metatable = LuaTable::new(vm);
        metatable.set_field(c"__index", src)?;
        metatable.set_field(c"__newindex", RFunction::wrap(__newindex))?;
        wrapper.set_metatable(metatable);
        Ok(wrapper)
    }
}

pub struct Table;

impl Lib for Table {
    const NAMESPACE: &'static str = "bp3d.util.table";

    fn load(&self, namespace: &mut Namespace) -> crate::vm::Result<()> {
        namespace.add([
            ("update", RFunction::wrap(update)),
            ("count", RFunction::wrap(count)),
            ("tostring", RFunction::wrap(to_string)),
            ("contains", RFunction::wrap(contains)),
            ("containsKey", RFunction::wrap(contains_key)),
            ("protect", RFunction::wrap(protect))
        ])
    }
}
