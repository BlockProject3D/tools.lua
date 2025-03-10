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

use std::ffi::CStr;
use std::marker::PhantomData;
use bp3d_util::simple_error;
use crate::ffi::laux::luaL_newmetatable;
use crate::ffi::lua::{lua_pushcclosure, lua_setfield, lua_settop, CFunction};
use crate::vm::util::{LuaType, TypeName};
use crate::vm::Vm;

simple_error! {
    pub Error {
        ArgsEmpty => "no arguments specified in userdata function, please add at least one argument matching the type of self",
        MutViolation(&'static CStr) => "violation of the unique type rule for mutable method {:?}",
        Gc => "__gc meta-method is reserved for internal use, if you need Vm access in drop, please use LuaDrop",
        AlreadyRegistered(&'static CStr) => "userdata with class name {:?} has already been registered"
    }
}

pub struct Function {
    is_mutable: bool,
    args: Vec<TypeName>,
    name: &'static CStr,
    func: CFunction
}

impl Function {
    pub fn new(name: &'static CStr, func: CFunction) -> Function {
        Function {
            is_mutable: false,
            args: Vec::new(),
            name,
            func
        }
    }

    pub fn mutable(&mut self) -> &mut Self {
        self.is_mutable = true;
        self
    }

    pub fn arg<T: LuaType>(&mut self) -> &mut Self {
        for ty in T::lua_type() {
            self.args.push(ty);
        }
        self
    }

    /// Checks and builds this userdata function
    ///
    /// # Safety
    ///
    /// All function arguments must be added through the arg function, if not calling this function
    /// is considered UB.
    pub unsafe fn build(&self) -> Result<(&'static CStr, CFunction), Error> {
        if self.args.is_empty() {
            return Err(Error::ArgsEmpty);
        }
        if self.name == c"__gc" {
            return Err(Error::Gc);
        }
        if self.is_mutable {
            let initial = &self.args[0];
            for v in self.args.iter().skip(1) {
                if initial == v {
                    return Err(Error::MutViolation(self.name));
                }
            }
        }
        Ok((self.name, self.func))
    }
}

pub struct Registry<'a, T> {
    vm: &'a Vm,
    useless: PhantomData<T>,
}

impl<'a, T: UserData> Registry<'a, T> {
    pub fn new(vm: &'a Vm) -> Result<Self, Error> {
        let res = unsafe { luaL_newmetatable(vm.as_ptr(), T::CLASS_NAME.as_ptr()) };
        if res != 1 {
            return Err(Error::AlreadyRegistered(T::CLASS_NAME));
        }
        Ok(Registry { vm, useless: PhantomData })
    }

    pub fn add_method(&self, name: &'static CStr, func: CFunction) {
        unsafe {
            lua_pushcclosure(self.vm.as_ptr(), func, 0);
            lua_setfield(self.vm.as_ptr(), -2, name.as_ptr());
        }
    }
}

impl<'a, T> Drop for Registry<'a, T> {
    fn drop(&mut self) {
        unsafe {
            // Pop the userdata metatable from the stack.
            lua_settop(self.vm.as_ptr(), -2);
        }
    }
}

pub trait UserData: Sized {
    const CLASS_NAME: &'static CStr;

    fn register(registry: &Registry<Self>) -> Result<(), Error>;
}

//TODO: Implement FromLua on UserData only when that userdata is also UserDataImmutable
//TODO: luaL_testudata to avoid unwinding with luaL_checkudata
pub unsafe trait UserDataImmutable: UserData {}
