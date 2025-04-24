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

use crate::ffi::laux::{luaL_checkudata, luaL_newmetatable};
use crate::ffi::lua::{
    lua_pushcclosure, lua_pushnil, lua_pushvalue, lua_setfield, lua_setmetatable, lua_settop,
    CFunction, State,
};
use crate::vm::userdata::{AddGcMethod, Error, LuaDrop, NameConvert, UserData};
use crate::vm::util::{LuaType, TypeName};
use crate::vm::value::IntoLua;
use crate::vm::Vm;
use bp3d_debug::{debug, warning};
use std::cell::OnceCell;
use std::ffi::CStr;
use std::marker::PhantomData;

//TODO: This should be a builder.
//TODO: The actual function structure should only contain name and CFunction.

pub struct Function {
    is_mutable: bool,
    args: Vec<TypeName>,
    name: &'static CStr,
    func: CFunction,
}

impl Function {
    pub fn new(name: &'static CStr, func: CFunction) -> Function {
        Function {
            is_mutable: false,
            args: Vec::new(),
            name,
            func,
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
        if self.name == c"__index" {
            return Err(Error::Index);
        }
        if self.name == c"__metatable" {
            return Err(Error::Metatable);
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

pub struct Registry<'a, T: UserData, C: NameConvert> {
    vm: &'a Vm,
    useless: PhantomData<T>,
    has_gc: OnceCell<()>,
    case: C,
}

impl<'a, T: UserData, C: NameConvert> Registry<'a, T, C> {
    /// Creates a new [Registry] from the given Vm.
    ///
    /// # Arguments
    ///
    /// * `vm`: the vm in which to register the userdata metatable.
    /// * `case`: the case converter to apply to each name to be registered.
    ///
    /// returns: Result<Registry<T>, Error>
    ///
    /// # Safety
    ///
    /// Running operations on the vm after calling this method is UB unless this [Registry] object
    /// is dropped.
    pub unsafe fn new(vm: &'a Vm, case: C) -> Result<Self, Error> {
        if align_of::<T>() > 8 {
            return Err(Error::Alignment(align_of::<T>()));
        }
        let res = unsafe { luaL_newmetatable(vm.as_ptr(), T::CLASS_NAME.as_ptr()) };
        if res != 1 {
            unsafe { lua_settop(vm.as_ptr(), -2) };
            return Err(Error::AlreadyRegistered(T::CLASS_NAME));
        }
        let reg = Registry {
            vm,
            useless: PhantomData,
            has_gc: OnceCell::new(),
            case,
        };
        reg.add_field(c"__metatable", T::CLASS_NAME.to_str().unwrap_unchecked())
            .unwrap_unchecked();
        Ok(reg)
    }

    pub fn add_field(&self, name: &'static CStr, value: impl IntoLua) -> Result<(), Error> {
        let num = value.into_lua(self.vm);
        if num > 1 {
            unsafe { lua_settop(self.vm.as_ptr(), -(num as i32) - 1) };
            return Err(Error::MultiValueField);
        }
        unsafe {
            lua_setfield(self.vm.as_ptr(), -2, self.case.name_convert(name).as_ptr());
        }
        Ok(())
    }

    pub fn add_method(&self, name: &'static CStr, func: CFunction) {
        unsafe {
            lua_pushcclosure(self.vm.as_ptr(), func, 0);
            if &name.to_bytes()[..2] == b"__" {
                lua_setfield(self.vm.as_ptr(), -2, name.as_ptr());
            } else {
                lua_setfield(self.vm.as_ptr(), -2, self.case.name_convert(name).as_ptr());
            }
        }
    }

    pub fn add_gc_method(&self) {
        if std::mem::needs_drop::<T>() {
            extern "C-unwind" fn run_drop<T: UserData>(l: State) -> i32 {
                unsafe {
                    let udata = luaL_checkudata(l, 1, T::CLASS_NAME.as_ptr()) as *mut T;
                    lua_pushnil(l);
                    lua_setmetatable(l, 1);
                    std::ptr::drop_in_place(udata);
                }
                0
            }
            self.add_method(c"__gc", run_drop::<T>);
            debug!({UD=?T::CLASS_NAME}, "Type registered with simple Drop");
        }
        self.has_gc.set(()).unwrap();
    }
}

impl<T: UserData + LuaDrop, C: NameConvert> Registry<'_, T, C> {
    pub fn add_gc_method_with_lua_drop(&self) {
        extern "C-unwind" fn run_lua_drop<T: UserData + LuaDrop>(l: State) -> i32 {
            unsafe {
                let udata = luaL_checkudata(l, 1, T::CLASS_NAME.as_ptr()) as *mut T;
                lua_pushnil(l);
                lua_setmetatable(l, 1);
                (*udata).lua_drop(&Vm::from_raw(l));
            }
            0
        }
        extern "C-unwind" fn run_lua_drop_full<T: UserData + LuaDrop>(l: State) -> i32 {
            unsafe {
                let udata = luaL_checkudata(l, 1, T::CLASS_NAME.as_ptr()) as *mut T;
                lua_pushnil(l);
                lua_setmetatable(l, 1);
                (*udata).lua_drop(&Vm::from_raw(l));
                std::ptr::drop_in_place(udata);
            }
            0
        }
        if std::mem::needs_drop::<T>() {
            self.add_method(c"__gc", run_lua_drop_full::<T>);
            debug!({UD=?T::CLASS_NAME}, "Type registered with Drop and LuaDrop");
        } else {
            self.add_method(c"__gc", run_lua_drop::<T>);
            debug!({UD=?T::CLASS_NAME}, "Type registered with LuaDrop");
        }
        self.has_gc.set(()).unwrap();
    }
}

pub struct AddGcMethodAuto<T>(PhantomData<T>);

impl<T> Default for AddGcMethodAuto<T> {
    fn default() -> Self {
        AddGcMethodAuto(PhantomData)
    }
}

impl<T: UserData + LuaDrop> AddGcMethod<T> for AddGcMethodAuto<T> {
    fn add_gc_method<C: NameConvert>(&self, reg: &Registry<T, C>) {
        reg.add_gc_method_with_lua_drop();
    }
}

impl<T: UserData> AddGcMethod<T> for &AddGcMethodAuto<T> {
    fn add_gc_method<C: NameConvert>(&self, reg: &Registry<T, C>) {
        reg.add_gc_method();
    }
}

impl<T: UserData, C: NameConvert> Drop for Registry<'_, T, C> {
    fn drop(&mut self) {
        if std::mem::needs_drop::<T>() && self.has_gc.get().is_none() {
            warning!("No __gc method registered on a drop userdata type!");
            // No __gc method found in object that needs it force add it before finishing it.
            self.add_gc_method();
        }
        unsafe {
            lua_pushvalue(self.vm.as_ptr(), -1);
            lua_setfield(self.vm.as_ptr(), -2, c"__index".as_ptr());
            // Pop the userdata metatable from the stack.
            lua_settop(self.vm.as_ptr(), -2);
        }
    }
}
