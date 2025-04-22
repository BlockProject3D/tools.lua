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

use crate::ffi::lua::{
    lua_error, lua_pushstring, lua_sethook, Debug, State, MASKCALL, MASKCOUNT, MASKLINE, MASKRET,
};
use crate::vm::core::interrupt::Error;
use crate::vm::RootVm;
use bp3d_debug::{error, warning};
use libc::{c_int, pthread_kill, pthread_self, pthread_t, SIGUSR1};
use std::mem::MaybeUninit;
use std::sync::{Mutex, Once};
use std::thread::ThreadId;
use std::time::Duration;

pub struct Signal {
    l: State,
    thread: ThreadId,
    th: pthread_t,
}

struct SigState {
    l: State,
    thread: ThreadId,
    return_chan: std::sync::mpsc::Sender<Result<(), Error>>,
    notify_chan: std::sync::mpsc::Sender<()>,
}

unsafe impl Send for SigState {}

static SIG_STATE: Mutex<Option<SigState>> = Mutex::new(None);

extern "C-unwind" fn lua_interrupt(l: State, _: Debug) {
    {
        let mut state = SIG_STATE.lock().unwrap();
        if let Some(sig) = state.take() {
            if let Err(e) = sig.notify_chan.send(()) {
                error!({error=?e}, "Failed to notify interrupt signal")
            }
        }
    }
    unsafe {
        lua_sethook(l, None, 0, 0);
        lua_pushstring(l, c"interrupted".as_ptr());
        lua_error(l);
    }
}

extern "C" fn signal_handler(_: c_int) {
    let res = SIG_STATE.try_lock();
    match res {
        Ok(v) => {
            if let Some(v) = &*v {
                let current_id = std::thread::current().id();
                if current_id != v.thread {
                    v.return_chan.send(Err(Error::IncorrectThread)).unwrap();
                    return;
                }
                // Run the hook 1 instruction later.
                unsafe {
                    lua_sethook(
                        v.l,
                        Some(lua_interrupt),
                        MASKCOUNT | MASKCALL | MASKLINE | MASKRET,
                        1,
                    )
                };
                v.return_chan.send(Ok(())).unwrap();
            }
        }
        Err(e) => {
            error!({error=?e}, "Attempt to interrupt a Vm while interrupting a different Vm");
        }
    }
}

static SIG_BOUND: Once = Once::new();

impl Signal {
    pub fn create(vm: &mut RootVm) -> Self {
        let th = unsafe { pthread_self() };
        let l = vm.as_ptr();
        let thread = std::thread::current().id();
        SIG_BOUND.call_once(|| {
            let mut sig: libc::sigaction = unsafe { MaybeUninit::zeroed().assume_init() };
            sig.sa_sigaction = signal_handler as _;
            let ret = unsafe { libc::sigaction(SIGUSR1, &sig as _, std::ptr::null_mut()) };
            assert_eq!(ret, 0);
        });
        Self { l, thread, th }
    }

    pub fn send(&self, duration: Duration) -> Result<(), Error> {
        let (send, recv) = std::sync::mpsc::channel();
        let (send2, recv2) = std::sync::mpsc::channel();
        {
            let mut lock = SIG_STATE
                .try_lock()
                .map_err(|_| Error::AlreadyInterrupting)?;
            *lock = Some(SigState {
                l: self.l,
                thread: self.thread,
                return_chan: send,
                notify_chan: send2,
            });
        }
        let ret = unsafe { pthread_kill(self.th, SIGUSR1) };
        if ret != 0 {
            return Err(Error::Unknown);
        }
        recv.recv().unwrap()?;
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
