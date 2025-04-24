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

use crate::ffi::lua::{lua_pushnil, lua_pushvalue, lua_replace, lua_settop, Type};
use crate::vm::error::{Error, TypeError};
use crate::vm::value::IntoLua;
use crate::vm::Vm;

/// Ensures the given lua value at index is of a specified type.
#[inline(always)]
pub fn ensure_type_equals(vm: &Vm, index: i32, expected: Type) -> crate::vm::Result<()> {
    let ty = unsafe { crate::ffi::lua::lua_type(vm.as_ptr(), index) };
    if ty == expected {
        //FIXME: likely branch
        Ok(())
    } else {
        Err(Error::Type(TypeError {
            expected,
            actual: ty,
        }))
    }
}

/// Ensures the given lua value at index is at the top of the stack.
/// If the value at index is not at the top of the stack, this function moves it to the top and
/// replaces the original index by a nil value.
#[inline(always)]
pub fn ensure_value_top(vm: &Vm, index: i32) {
    let index = vm.get_absolute_index(index);
    if index != vm.top() {
        let l = vm.as_ptr();
        unsafe {
            lua_pushvalue(l, index);
            lua_pushnil(l);
            lua_replace(l, index); // Replace the value at index by a nil.
        }
    }
}

/// Ensures a single value is pushed onto the lua stack, this function automatically reverts the
/// stack if value pushed more than 1 element onto the stack.
///
/// # Arguments
///
/// * `vm`: the vm to operate on.
/// * `value`: the value to be placed on the lua stack.
pub fn ensure_single_into_lua(vm: &Vm, value: impl IntoLua) -> crate::vm::Result<()> {
    let nums = value.into_lua(vm);
    if nums != 1 {
        // Clear the stack.
        unsafe { lua_settop(vm.as_ptr(), -(nums as i32) - 1) };
        return Err(Error::MultiValue);
    }
    Ok(())
}
