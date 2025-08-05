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

use super::{Error, InterruptibleRootVm};
use crate::ffi::ext::lua_ext_ccatch_error;
use crate::ffi::lua::{
    lua_pushstring, lua_sethook, Debug, State, MASKCALL, MASKCOUNT, MASKLINE, MASKRET,
};
use bp3d_debug::{error, warning};
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Diagnostics::Debug::{GetThreadContext, CONTEXT};
use windows_sys::Win32::System::Threading::{GetCurrentThread, ResumeThread, SuspendThread};

static SIG_STATE: Mutex<Option<std::sync::mpsc::Sender<()>>> = Mutex::new(None);

extern "C-unwind" fn lua_interrupt(l: State, _: Debug) {
    {
        let mut state = SIG_STATE.lock().unwrap();
        if let Some(sig) = state.take() {
            if let Err(e) = sig.send(()) {
                error!({error=?e}, "Failed to notify interrupt signal")
            }
        }
    }
    unsafe {
        lua_sethook(l, None, 0, 0);
        lua_pushstring(l, c"interrupted".as_ptr());
        lua_ext_ccatch_error(l);
    }
}

pub struct Signal {
    l: State,
    th: HANDLE,
    alive: Arc<AtomicBool>,
}

impl Signal {
    pub fn create(vm: &mut InterruptibleRootVm) -> Self {
        let alive = InterruptibleRootVm::get_alive(vm).clone();
        let th = unsafe { GetCurrentThread() };
        let l = vm.as_ptr();
        Self { l, th, alive }
    }

    pub fn send(&self, duration: Duration) -> Result<(), Error> {
        if !self.alive.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }
        let (send2, recv2) = std::sync::mpsc::channel();
        {
            let mut lock = SIG_STATE
                .try_lock()
                .map_err(|_| Error::AlreadyInterrupting)?;
            *lock = Some(send2);
        }
        if self.th == unsafe { GetCurrentThread() } {
            // If somehow the system thread that ineterrupts the Vm is the same as the one which started the Vm, then directly set the hook.
            unsafe {
                lua_sethook(
                    self.l,
                    Some(lua_interrupt),
                    MASKCOUNT | MASKCALL | MASKLINE | MASKRET,
                    1,
                );
            }
        } else {
            unsafe {
                let mut ctx: CONTEXT = std::mem::zeroed();
                // Requests to suspend the thread.
                if SuspendThread(self.th) == u32::MAX {
                    //(DWORD) -1
                    return Err(Error::Unknown);
                }
                // This call forces synchronization with the thread to be suspended.
                if GetThreadContext(self.th, &mut ctx as _) == 0 {
                    return Err(Error::Unknown);
                }
                lua_sethook(
                    self.l,
                    Some(lua_interrupt),
                    MASKCOUNT | MASKCALL | MASKLINE | MASKRET,
                    1,
                );
                // Resume the thread.
                let _ = ResumeThread(self.th);
            }
        }
        match recv2.recv_timeout(duration) {
            Ok(()) => Ok(()),
            Err(e) => {
                warning!({error=?e}, "Error attempting to wait for interrupt notification");
                {
                    let mut guard = SIG_STATE.lock().unwrap();
                    *guard = None;
                }
                Err(Error::Timeout)
            }
        }
    }
}
