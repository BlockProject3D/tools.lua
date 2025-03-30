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

use crate::vm::Vm;

pub trait FromLua<'a>: Sized {
    /// Reads the value at the specified index in the given [Vm].
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] to read from.
    /// * `index`: the index at which to try reading the value from. The index does not have to be
    ///            absolute.
    ///
    /// returns: Result<Self, Error>
    ///
    /// # Safety
    ///
    /// This function assumes the type of the value at index `index` is already of the expected type,
    /// if not, calling this function is UB.
    unsafe fn from_lua_unchecked(vm: &'a Vm, index: i32) -> Self;

    /// Attempt to read the value at the specified index in the given [Vm].
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] to read from.
    /// * `index`: the index at which to try reading the value from. The index does not have to be
    ///            absolute.
    ///
    /// returns: Result<Self, Error>
    fn from_lua(vm: &'a Vm, index: i32) -> crate::vm::Result<Self>;

    /// Returns the number of values to be expected on the lua stack, after reading this value.
    #[inline(always)]
    fn num_values() -> u16 {
        1
    }
}

/// This trait represents a value convertible to lua outside Rust function calls. For lua values
/// returned by Rust functions, see [IntoParam](crate::vm::function::IntoParam).
///
/// # Safety
///
/// When implementing this trait, ensure that the number returned by
/// [into_lua](IntoLua::into_lua) is EXACTLY equal to the number of values pushed onto the lua
/// stack. If more or fewer than advertised values exists on the stack after the call then the impl
/// is considered UB.
pub unsafe trait IntoLua: Sized {
    /// Attempt to push self onto the top of the stack in the given [Vm].
    ///
    /// Returns the number values pushed into the lua stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] to push into.
    ///
    /// returns: u16 number of elements pushed onto the Lua stack.
    fn into_lua(self, vm: &Vm) -> u16;
}
