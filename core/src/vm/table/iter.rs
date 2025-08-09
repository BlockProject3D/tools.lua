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

use crate::ffi::lua::{lua_next, lua_pushnil, lua_settop};
use crate::vm::value::any::Any;
use crate::vm::value::FromLua;
use crate::vm::Vm;

pub struct Iter<'a> {
    vm: &'a Vm,
    index: i32,
    has_ended: bool,
    has_started: bool,
    last_top: i32,
}

impl<'a> Iter<'a> {
    pub(super) fn from_raw(vm: &'a Vm, index: i32) -> Self {
        // Push a nil value on the stack to allow the iterator to work.
        unsafe { lua_pushnil(vm.as_ptr()) };
        Self {
            vm,
            index,
            has_ended: false,
            has_started: false,
            last_top: vm.top(),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = crate::vm::Result<(Any<'a>, Any<'a>)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_started {
            // This ensures the iterator remains safe.
            if self.vm.top() != self.last_top {
                panic!(
                    "Attempt to iterate on moved values (expected Vm top: {}, got: {})",
                    self.last_top,
                    self.vm.top()
                );
            }
            // Pop the last value on the stack which corresponds to the last value from lua_next.
            // Only if the iterator was started.
            unsafe { lua_settop(self.vm.as_ptr(), -2) };
        }
        let ret = unsafe { lua_next(self.vm.as_ptr(), self.index) };
        self.last_top = self.vm.top();
        self.has_started = true;
        if ret != 0 {
            let value = Any::from_lua(self.vm, -2);
            let key = Any::from_lua(self.vm, -1);
            Some(match (value, key) {
                (Ok(key), Ok(value)) => Ok((key, value)),
                (Ok(_), Err(e)) => Err(e),
                (Err(e), Ok(_)) => Err(e),
                (Err(_), Err(e)) => Err(e),
            })
        } else {
            self.has_ended = true;
            None
        }
    }
}

impl Drop for Iter<'_> {
    fn drop(&mut self) {
        if !self.has_ended {
            // If the iterator did not reach the end, clear key-value pair from the lua stack.
            unsafe { lua_settop(self.vm.as_ptr(), -3) };
        }
    }
}
