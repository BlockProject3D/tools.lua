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

/// This trait represents a generic key which can be constructed from an index on the lua stack.
pub trait FromIndex {
    /// Constructs a new instance of this generic key from the given vm and index.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] instance to manipulate.
    /// * `index`: the index of the value on the lua stack.
    ///
    /// returns: Self
    ///
    /// # Safety
    ///
    /// This function removes the value at index `index` and so assumes no more references exists
    /// to it, failure to ensure this is UB.
    unsafe fn from_index(vm: &Vm, index: i32) -> Self;

}

/// This trait represents a generic key which can be set from an index on the lua stack.
pub trait Set {
    /// Sets the value of this generic key from the given vm and index.
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] instance to manipulate.
    /// * `index`: the index of the value on the lua stack.
    ///
    /// returns: ()
    ///
    /// # Safety
    ///
    /// This function removes the value at index `index` and so assumes no more references exists
    /// to it, failure to ensure this is UB. The function also assumes this generic key still
    /// exists in the registry table. Finally, this assumes this key does not conflict with a
    /// different one.
    unsafe fn set(&self, vm: &Vm, index: i32);
}

pub trait Value: 'static {
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
    unsafe fn from_registry(vm: &Vm, index: i32) -> Self::Value<'_>;

    /// Intializes a new generic key from the given value.
    ///
    /// This function should call R::from_index with a matching index and [Vm] instance.
    fn push_registry<R: FromIndex>(value: Self::Value<'_>) -> R;

    /// Assign this value to the given generic registry key.
    ///
    /// This function should call key.set with a matching index and [Vm] instance.
    ///
    /// # Arguments
    ///
    /// * `key`: the key to update.
    /// * `value`: the new value.
    ///
    /// returns: ()
    ///
    /// # Safety
    ///
    /// This function assumes the generic key still exists in the registry table.
    unsafe fn set_registry(key: &impl Set, value: Self::Value<'_>);
}
