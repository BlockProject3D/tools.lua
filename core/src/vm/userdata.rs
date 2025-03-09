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

use std::any::TypeId;
use std::ffi::CStr;
use crate::ffi::lua::CFunction;
use crate::vm::util::LuaType;

pub struct Function {
    is_mutable: bool,
    args: Vec<TypeId>,
    func: CFunction
}

impl Function {
    pub fn new(func: CFunction) -> Function {
        Function {
            is_mutable: false,
            args: Vec::new(),
            func
        }
    }

    pub fn mutable(&mut self) -> &mut Self {
        self.is_mutable = true;
        self
    }

    pub fn arg<T: LuaType>(&mut self) -> &mut Self {
        self.args.push(T::lua_type());
        self
    }
}

pub trait UserData: Sized {
    const CLASS_NAME: &'static CStr;

    //fn register(registry: &Registry<Self>);
}

//TODO: Implement FromLua on UserData only when that userdata is also UserDataImmutable
//TODO: most likely need another luajit hack to allow returning errors from luaL_checkudata instead
// of unwinding
pub unsafe trait UserDataImmutable: UserData {}
