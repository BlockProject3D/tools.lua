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
use crate::util::Namespace;
use crate::vm::core::debug::DebugRegistry;
use crate::vm::core::iter::start;
use crate::vm::function::types::RFunction;
use crate::vm::table::Table;
use crate::vm::value::any::Any;

decl_lib_func! {
    fn dump_stack(vm: &Vm, start_index: i32) -> crate::vm::Result<Table> {
        let mut tbl = Table::new(vm);
        let iter = start::<Any>(vm, start_index);
        for value in iter {
            match value {
                Ok(v) => tbl.push(v.to_string())?,
                Err(e) => tbl.push(e.to_string())?,
            }
        }
        Ok(tbl)
    }
}

decl_lib_func! {
    fn dump_libs(vm: &Vm) -> crate::vm::Result<Table> {
        let mut tbl = Table::new(vm);
        if let Some(vv) = DebugRegistry::list(vm, crate::vm::core::debug::Lib) {
            for v in vv {
                tbl.push(v)?;
            }
        }
        Ok(tbl)
    }
}

decl_lib_func! {
    fn dump_classes(vm: &Vm) -> crate::vm::Result<Table> {
        let mut tbl = Table::new(vm);
        if let Some(vv) = DebugRegistry::list(vm, crate::vm::core::debug::Class) {
            for v in vv {
                tbl.push(v)?;
            }
        }
        Ok(tbl)
    }
}

//TODO: debugger to dump userdata metatable, static table and namespace content

pub struct Debug;

impl Lib for Debug {
    const NAMESPACE: &'static str = "bp3d.lua.debug";

    fn load(&self, namespace: &mut Namespace) -> crate::vm::Result<()> {
        namespace.add([
            ("dumpStack", RFunction::wrap(dump_stack)),
            ("dumpLibs", RFunction::wrap(dump_libs)),
            ("dumpClasses", RFunction::wrap(dump_classes))
        ])
    }
}
