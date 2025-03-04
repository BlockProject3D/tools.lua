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

use std::cell::Cell;
use crate::ffi::laux::{luaL_newstate, luaL_openlibs};
use crate::ffi::lua::State;

pub struct Stack {
    l: State,
    index: Cell<i32>
}

impl Stack {
    /// Creates a new [Stack] by wrapping an existing lua VM.
    ///
    /// # Arguments
    ///
    /// * `l`: the raw lua state to wrap.
    /// * `start`: the index at which to start reading values from the lua stack.
    ///
    /// returns: Stack
    ///
    /// # Safety
    ///
    /// This struct SHALL only exist in a [CFunction](crate::ffi::lua::CFunction). Usage in any other
    /// context is UB.
    pub unsafe fn wrap(l: State, start: i32) -> Stack {
        Stack {
            l,
            index: Cell::new(start)
        }
    }

    pub fn as_ptr(&self) -> State {
        self.l
    }

    pub fn pop(&self) -> i32 {
        let i = self.index.get();
        self.index.set(i + 1);
        i
    }
}

pub struct Vm {
    l: State
}

impl Vm {
    pub fn new() -> Vm {
        let l = unsafe { luaL_newstate() };
        unsafe { luaL_openlibs(l) };
        Vm {
            l
        }
    }

    pub fn as_ptr(&self) -> State {
        self.l
    }
}
