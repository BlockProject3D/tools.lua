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
use bp3d_debug::info;
use bp3d_lua::decl_closure;
use bp3d_lua::libs::Lib;
use bp3d_lua::util::Namespace;
use bp3d_lua::vm::closure::rc::{Rc, Shared};
use crate::data::DataOut;
use crate::lib1::autocomplete_api::{build_completions, delete_completions};
use crate::lib1::scheduler_api::{schedule_in, schedule_periodically};
use crate::scheduler::SchedulerPtr;

mod autocomplete_api;
mod scheduler_api;

decl_closure! {
    fn request_exit |running: Rc<Cell<bool>>| () -> () {
        info!("Lua has requested exit");
        running.set(false);
    }
}

pub struct Shell {
    log_ch: Shared<DataOut>,
    scheduler: Shared<SchedulerPtr>,
    running: Shared<Cell<bool>>
}

impl Shell {
    pub fn new(log_ch: DataOut, scheduler: Shared<SchedulerPtr>, running: Shared<Cell<bool>>) -> Shell {
        Self {
            log_ch: log_ch.into(),
            scheduler,
            running
        }
    }
}

impl Lib for Shell {
    const NAMESPACE: &'static str = "bp3d.lua.shell";

    fn load(&self, namespace: &mut Namespace) -> bp3d_lua::vm::Result<()> {
        let rc = Rc::from_rust(namespace.vm(), self.log_ch.clone());
        let rc1 = Rc::from_rust(namespace.vm(), self.log_ch.clone());
        let r1 = Rc::from_rust(namespace.vm(), self.scheduler.clone());
        let r2 = Rc::from_rust(namespace.vm(), self.scheduler.clone());
        let running = Rc::from_rust(namespace.vm(), self.running.clone());
        namespace.add([
            ("buildCompletions", build_completions(rc)),
            ("deleteCompletions", delete_completions(rc1))
        ])?;
        namespace.add([
            ("scheduleIn", schedule_in(r1)),
            ("schedulePeriodically", schedule_periodically(r2))
        ])?;
        namespace.add([("requestExit", request_exit(running))])
    }
}
