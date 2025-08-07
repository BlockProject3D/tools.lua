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

use bp3d_debug::debug;
use crate::ffi::laux::{luaL_newstate, luaL_openlibs};
use crate::ffi::lua::lua_close;
use crate::vm::core::destructor::Pool;
use crate::vm::registry::named::{handle_root_vm_init, handle_root_vm_uninit};
use crate::vm::Vm;

#[cfg(not(feature = "send"))]
thread_local! {
    // WTF?! The compiler should be smart enough to do this on its own! Another compiler defect!
    static HAS_VM: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[repr(transparent)]
pub struct UnsafeRootVm(pub Vm);

impl UnsafeRootVm {
    pub fn new(is_send: bool) -> UnsafeRootVm {
        #[cfg(not(feature = "send"))]
        if HAS_VM.get() {
            panic!("A VM already exists for this thread.")
        }
        let l = unsafe { luaL_newstate() };
        unsafe { luaL_openlibs(l) };
        #[cfg(not(feature = "send"))]
        HAS_VM.set(true);
        let mut vm = UnsafeRootVm(unsafe { Vm::from_raw(l) });
        handle_root_vm_init();
        unsafe { Pool::new_in_vm(&mut vm.0, is_send) };
        vm
    }
}

impl Drop for UnsafeRootVm {
    fn drop(&mut self) {
        debug!("Deleting destructor pool");
        unsafe {
            drop(Box::from_raw(Pool::from_vm(&mut self.0)));
        }
        handle_root_vm_uninit();
        unsafe {
            debug!("Closing Lua VM...");
            lua_close(self.0.as_ptr());
        }
        #[cfg(not(feature = "send"))]
        HAS_VM.set(false);
    }
}
