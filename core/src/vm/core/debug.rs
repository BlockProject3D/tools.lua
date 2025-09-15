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

use crate::vm::core::destructor::Pool;
use crate::vm::registry::lua_ref::LuaRef as LiveLuaRef;
use crate::vm::registry::named::Key;
use crate::vm::registry::types::LuaRef;
use crate::vm::userdata::UserData;
use crate::vm::value::types::RawPtr;
use crate::vm::Vm;
use std::collections::HashMap;

pub trait DebugItemType {
    const NAME: &'static str;
}

pub struct Lib;
pub struct Class;

impl DebugItemType for Lib {
    const NAME: &'static str = "Lib";
}

impl DebugItemType for Class {
    const NAME: &'static str = "Class";
}

pub trait DebugItem<T: DebugItemType> {
    fn describe() -> String;
}

#[cfg(feature = "libs-core")]
impl<T: crate::libs::Lib + ?Sized> DebugItem<Lib> for T {
    fn describe() -> String {
        let lib_name = std::any::type_name::<T>();
        let lib_namespace = T::NAMESPACE;
        let desc = format!("{}: {}", lib_name, lib_namespace);
        desc
    }
}

impl<T: UserData> DebugItem<Class> for T {
    fn describe() -> String {
        T::FULL_TYPE.to_string_lossy().into()
    }
}

static DBG_REG: Key<LuaRef<RawPtr<DebugRegistry>>> = Key::new("__debug_registry__");

pub struct DebugRegistry {
    map: HashMap<&'static str, Vec<String>>,
}

impl DebugRegistry {
    fn add_internal<D: DebugItem<T> + ?Sized, T: DebugItemType>(&mut self) {
        self.map
            .entry(T::NAME)
            .or_insert_with(Vec::new)
            .push(D::describe());
    }

    fn list_internal<T: DebugItemType>(&self, _: T) -> Option<Vec<String>> {
        self.map.get(T::NAME).cloned()
    }

    fn get(vm: &Vm) -> RawPtr<DebugRegistry> {
        if let None = DBG_REG.push(vm) {
            let ptr = Pool::attach_send(
                vm,
                Box::new(DebugRegistry {
                    map: HashMap::new(),
                }),
            );
            DBG_REG.set(LiveLuaRef::new(vm, RawPtr::new(ptr)));
        }
        let ptr = DBG_REG.push(vm).unwrap().get();
        ptr
    }

    pub fn add<D: DebugItem<T> + ?Sized, T: DebugItemType>(vm: &Vm) {
        let ptr = Self::get(vm);
        unsafe { (*ptr.as_mut_ptr()).add_internal::<D, T>() };
    }

    pub fn list(vm: &Vm, ty: impl DebugItemType) -> Option<Vec<String>> {
        let ptr = Self::get(vm);
        unsafe { (*ptr.as_ptr()).list_internal(ty) }
    }
}
