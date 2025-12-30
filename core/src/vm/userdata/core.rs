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
    lua_getmetatable, lua_pushcclosure, lua_pushlightuserdata, lua_pushnil, lua_pushvalue,
    lua_rawget, lua_setfield, lua_setmetatable, lua_settop, lua_touserdata, lua_type, CFunction,
    State, Type, GLOBALSINDEX,
};
use crate::vm::table::Table;
use crate::vm::userdata::{AddGcMethod, Error, LuaDrop, NameConvert, UserData};
use crate::vm::util::{LuaType, TypeName};
use crate::vm::value::IntoLua;
use crate::vm::Vm;
use bp3d_debug::{debug, trace, warning};
use std::cell::OnceCell;
use std::ffi::{c_void, CStr};
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct Function {
    pub name: &'static CStr,
    pub func: CFunction,
}

pub struct Builder {
    is_mutable: bool,
    args: Vec<TypeName>,
    f: Function,
}

impl Builder {
    pub fn new(name: &'static CStr, func: CFunction) -> Builder {
        Builder {
            is_mutable: false,
            args: Vec::new(),
            f: Function { name, func },
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
    pub unsafe fn build(&self) -> Result<Function, Error> {
        if self.args.is_empty() {
            return Err(Error::ArgsEmpty);
        }
        if self.f.name == c"__gc" {
            return Err(Error::Gc);
        }
        if self.f.name == c"__metatable" {
            return Err(Error::Metatable);
        }
        if self.is_mutable {
            let initial = &self.args[0];
            for v in self.args.iter().skip(1) {
                if v == &TypeName::Some("function")
                    || v == &TypeName::Some("table")
                    || v == &TypeName::Some("userdata")
                {
                    // Forbid functions, tables and userdata in mutable userdata types.
                    // This is to ensure no mutable userdata may call back into itself.
                    return Err(Error::MutViolation(self.f.name));
                }
                if initial == v {
                    return Err(Error::MutViolation(self.f.name));
                }
            }
        }
        Ok(self.f)
    }
}

pub struct Registry<'a, T: UserData, C: NameConvert> {
    vm: &'a Vm,
    useless: PhantomData<T>,
    has_gc: OnceCell<()>,
    has_index: OnceCell<()>,
    has_static: OnceCell<()>,
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
        Table::new(vm);
        let res = unsafe { luaL_newmetatable(vm.as_ptr(), T::FULL_TYPE.as_ptr()) };
        // Pop the userdata metatable alongside its statics table from the stack.
        if res != 1 {
            unsafe { lua_settop(vm.as_ptr(), -3) };
            return Err(Error::AlreadyRegistered(T::CLASS_NAME));
        }
        let reg = Registry {
            vm,
            useless: PhantomData,
            has_gc: OnceCell::new(),
            has_index: OnceCell::new(),
            has_static: OnceCell::new(),
            case,
        };
        reg.add_field(c"__metatable", T::CLASS_NAME.to_str().unwrap_unchecked())
            .unwrap_unchecked();
        Ok(reg)
    }

    fn add_field(&self, name: &'static CStr, value: impl IntoLua) -> Result<(), Error> {
        trace!("Set metatable field {:?}", name);
        let num = value.into_lua(self.vm);
        if num > 1 {
            unsafe { lua_settop(self.vm.as_ptr(), -(num as i32) - 1) };
            return Err(Error::MultiValueField);
        }
        unsafe {
            lua_setfield(self.vm.as_ptr(), -2, name.as_ptr());
        }
        Ok(())
    }

    pub fn add_static_field(&self, name: &'static CStr, value: impl IntoLua) -> Result<(), Error> {
        let _ = self.has_static.set(());
        let mut static_table = unsafe { Table::from_raw(self.vm, -3) };
        static_table
            .set(&*self.case.name_convert(name), value)
            .map_err(|_| Error::MultiValueField)?;
        Ok(())
    }

    fn add_index_metamethod(&self, f: CFunction) {
        warning!({UD=?T::CLASS_NAME}, "Overriding __index on an UserData object may worsen performance of method calls by introducing an additional indirection on the __index metamethod");
        extern "C-unwind" fn __index(l: State) -> i32 {
            unsafe {
                lua_getmetatable(l, 1);
                lua_pushvalue(l, 2);
                lua_rawget(l, -2);
                let ty = lua_type(l, -1);
                if ty != Type::Nil {
                    return 1;
                }
                // Pop both the metatatble and the rawget value from the stack before running the
                // custom __index method.
                lua_settop(l, -3);
                let f: CFunction = std::mem::transmute(lua_touserdata(l, GLOBALSINDEX - 1));
                f(l)
            }
        }
        unsafe {
            lua_pushlightuserdata(self.vm.as_ptr(), f as *mut c_void);
            lua_pushcclosure(self.vm.as_ptr(), __index, 1);
            lua_setfield(self.vm.as_ptr(), -2, c"__index".as_ptr());
        }
        self.has_index.set(()).unwrap();
    }

    pub fn add_method(&self, f: Function) {
        if &f.name.to_bytes() == b"__index" {
            self.add_index_metamethod(f.func);
            return;
        }
        unsafe {
            lua_pushcclosure(self.vm.as_ptr(), f.func, 0);
            if &f.name.to_bytes()[..2] == b"__" {
                lua_setfield(self.vm.as_ptr(), -2, f.name.as_ptr());
            } else {
                lua_setfield(
                    self.vm.as_ptr(),
                    -2,
                    self.case.name_convert(f.name).as_ptr(),
                );
            }
        }
    }

    pub fn add_gc_method(&self) {
        if std::mem::needs_drop::<T>() {
            extern "C-unwind" fn run_drop<T: UserData>(l: State) -> i32 {
                unsafe {
                    let udata = luaL_checkudata(l, 1, T::FULL_TYPE.as_ptr()) as *mut T;
                    lua_pushnil(l);
                    lua_setmetatable(l, 1);
                    std::ptr::drop_in_place(udata);
                }
                0
            }
            self.add_method(Function {
                name: c"__gc",
                func: run_drop::<T>,
            });
            debug!({UD=?T::CLASS_NAME}, "Type registered with simple Drop");
        } else {
            debug!({UD=?T::CLASS_NAME}, "Type does not need any drop behavior");
        }
        self.has_gc.set(()).unwrap();
    }
}

impl<T: UserData + LuaDrop, C: NameConvert> Registry<'_, T, C> {
    pub fn add_gc_method_with_lua_drop(&self) {
        extern "C-unwind" fn run_lua_drop<T: UserData + LuaDrop>(l: State) -> i32 {
            unsafe {
                let udata = luaL_checkudata(l, 1, T::FULL_TYPE.as_ptr()) as *mut T;
                lua_pushnil(l);
                lua_setmetatable(l, 1);
                (*udata).lua_drop(&Vm::from_raw(l));
            }
            0
        }
        extern "C-unwind" fn run_lua_drop_full<T: UserData + LuaDrop>(l: State) -> i32 {
            unsafe {
                let udata = luaL_checkudata(l, 1, T::FULL_TYPE.as_ptr()) as *mut T;
                lua_pushnil(l);
                lua_setmetatable(l, 1);
                (*udata).lua_drop(&Vm::from_raw(l));
                std::ptr::drop_in_place(udata);
            }
            0
        }
        if std::mem::needs_drop::<T>() {
            self.add_method(Function {
                name: c"__gc",
                func: run_lua_drop_full::<T>,
            });
            debug!({UD=?T::CLASS_NAME}, "Type registered with Drop and LuaDrop");
        } else {
            self.add_method(Function {
                name: c"__gc",
                func: run_lua_drop::<T>,
            });
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
            warning!("No __gc method registered on a userdata type which needs drop!");
            // No __gc method found in object that needs it force add it before finishing it.
            self.add_gc_method();
        }
        unsafe {
            if self.has_index.get().is_none() {
                lua_pushvalue(self.vm.as_ptr(), -1);
                lua_setfield(self.vm.as_ptr(), -2, c"__index".as_ptr());
            }
            if self.has_static.get().is_some() {
                lua_pushvalue(self.vm.as_ptr(), -2); // Push the static table.
                lua_setfield(self.vm.as_ptr(), -2, c"__static".as_ptr());
            }
            // Pop the userdata metatable alongside its statics table from the stack.
            lua_settop(self.vm.as_ptr(), -3);
        }
    }
}
