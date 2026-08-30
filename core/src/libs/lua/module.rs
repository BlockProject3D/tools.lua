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
use crate::util::module::Result;
use crate::util::module::{Error, ModuleManager};
use crate::util::Namespace;
use crate::vm::core::destructor::Pool;
use crate::vm::function::types::RFunction;
use crate::vm::registry::named::Key;
use crate::vm::registry::types::LuaRef;
use crate::vm::value::types::RawPtr;
use crate::vm::Vm;

static KEY: Key<LuaRef<RawPtr<ModuleManager>>> = Key::new("__module_manager__");

fn register_module_manager(vm: &Vm) {
    let value = KEY.push(vm);
    if value.is_some() {
        panic!("A ModuleManager is already registered in the current Vm");
    }
    let manager = ModuleManager::new();
    let mut value = Box::new(manager);
    let value2 = &mut *value as *mut ModuleManager;
    Pool::attach_post_close(vm, || {
        let value = value;
        drop(value);
    });
    let value = crate::vm::registry::lua_ref::LuaRef::new(vm, RawPtr::new(value2));
    KEY.set(value);
}

decl_lib_func! {
    fn load_module(vm: &Vm, lib: &str, plugin: &str) -> Result<()> {
        let manager = KEY.push(vm);
        if let Some(manager) = manager {
            unsafe { (&mut *manager.get().as_mut_ptr()).load(lib, plugin, vm) }?;
            Ok(())
        } else {
            Err(Error::NotRegistered)
        }
    }
}

pub struct Module;

impl Lib for Module {
    const NAMESPACE: &'static str = "bp3d.lua.module";

    fn load(&self, namespace: &mut Namespace) -> crate::vm::Result<()> {
        register_module_manager(namespace.vm());
        namespace.add([("load", RFunction::wrap(load_module))])
    }
}
