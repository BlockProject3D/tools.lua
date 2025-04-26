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
use std::ops::{Deref, DerefMut};
use bp3d_debug::debug;
use crate::ffi::laux::{luaL_newstate, luaL_openlibs};
use crate::ffi::lua::lua_close;
use crate::vm::core::destructor::Pool;
use crate::vm::Vm;

thread_local! {
    // WTF?! The compiler should be smart enough to do this on its own! Another compiler defect!
    static HAS_VM: Cell<bool> = const { Cell::new(false) };
}

pub struct RootVm {
    vm: Vm
}

impl Default for RootVm {
    fn default() -> Self {
        Self::new()
    }
}

impl RootVm {
    pub fn new() -> RootVm {
        if HAS_VM.get() {
            panic!("A VM already exists for this thread.")
        }
        let l = unsafe { luaL_newstate() };
        unsafe { luaL_openlibs(l) };
        HAS_VM.set(true);
        let mut vm = RootVm {
            vm: unsafe { Vm::from_raw(l) },
        };
        unsafe { Pool::new_in_vm(&mut vm) };
        vm
    }
}

impl Deref for RootVm {
    type Target = Vm;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.vm
    }
}

impl DerefMut for RootVm {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vm
    }
}

impl Drop for RootVm {
    fn drop(&mut self) {
        debug!("Deleting destructor pool");
        unsafe {
            drop(Box::from_raw(Pool::from_vm(self)));
        }
        unsafe {
            debug!("Closing Lua VM...");
            lua_close(self.vm.as_ptr());
        }
        HAS_VM.set(false);
    }
}

