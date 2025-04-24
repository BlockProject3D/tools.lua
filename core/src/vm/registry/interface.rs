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

use crate::vm::registry::core::RegistryKey;
use crate::vm::Vm;

//TODO: Try to find a better name.

pub trait RegistryValue: 'static {
    type Value<'a>;

    /// Reads the upvalue at the given location on the lua stack.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] to read from.
    /// * `index`: the index of the value. This index is not guaranteed to be absolute.
    ///
    /// returns: Self::Value
    ///
    /// # Safety
    ///
    /// This function assumes the value at the top of the stack is of type `Self`. This function is
    /// UB otherwise.
    unsafe fn to_lua_value(vm: &Vm, index: i32) -> Self::Value<'_>;
}

/// A trait to produce registry values safely.
pub trait Registry: Sized {
    type RegistryValue: RegistryValue;

    /// Register this value into the registry.
    ///
    /// returns: RegistryKey<Self::RegistryValue>
    fn registry_put(self) -> RegistryKey<Self::RegistryValue>;

    /// Swaps the value pointed by `old` in the registry to this value.
    ///
    /// # Arguments
    ///
    /// * `old`: the old registry key to be replaced.
    fn registry_swap(
        self,
        old: RegistryKey<Self::RegistryValue>,
    ) -> RegistryKey<Self::RegistryValue>;
}
