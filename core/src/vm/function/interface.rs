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

use crate::util::SimpleDrop;
use crate::vm::Vm;
use crate::vm::util::LuaType;

/// This trait represents a function return value.
///
/// # Safety
///
/// When implementing this trait, ensure that the number returned by
/// [into_param](IntoParam::into_param) is EXACTLY equal to the number of values pushed onto the lua
/// stack. If more or fewer than advertised values exists on the stack after the call then the impl
/// is considered UB.
pub unsafe trait IntoParam: Sized {
    /// Turns self into a function return parameter.
    ///
    /// This function returns the number of parameters pushed onto the lua stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] to push this value to.
    ///
    /// returns: u16
    fn into_param(self, vm: &Vm) -> u16;
}

/// This trait represents a function parameter.
pub trait FromParam<'a>: Sized + SimpleDrop + LuaType {
    /// Reads this value from the given lua stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] to read from.
    /// * `index`: index of the parameter to read. This index is guaranteed to be absolute.
    ///
    /// returns: Self
    ///
    /// # Safety
    ///
    /// Calling this function outside the body of a [CFunction](crate::ffi::lua::CFunction) is UB.
    /// Calling this function in a non-POF segment of that CFunction is also UB. If the index is not
    /// absolute, this function may cause UB.
    unsafe fn from_param(vm: &'a Vm, index: i32) -> Self;
}
