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

use bp3d_lua::decl_closure;
use bp3d_lua::libs::Lib;
use bp3d_lua::util::Namespace;
use bp3d_lua::vm::closure::rc::Rc;
use bp3d_lua::vm::thread::value::Value;
use crate::scheduler::SchedulerPtr;

decl_closure! {
    fn schedule_in |scheduler: Rc<SchedulerPtr>| (thread: Value, after_ms: u32) -> () {
        scheduler.schedule_in(thread, after_ms);
    }
}

decl_closure! {
    fn schedule_periodically |scheduler: Rc<SchedulerPtr>| (thread: Value, period_ms: u32) -> () {
        scheduler.schedule_periodically(thread, period_ms);
    }
}

pub struct SchedulerApi(std::rc::Rc<SchedulerPtr>);

impl SchedulerApi {
    pub fn new(ptr: std::rc::Rc<SchedulerPtr>) -> Self {
        Self(ptr)
    }
}

impl Lib for SchedulerApi {
    const NAMESPACE: &'static str = "bp3d.lua.shell";

    fn load(&self, namespace: &mut Namespace) -> bp3d_lua::vm::Result<()> {
        let r1 = Rc::from_rust(namespace.vm(), self.0.clone());
        let r2 = Rc::from_rust(namespace.vm(), self.0.clone());
        namespace.add([
            ("scheduleIn", schedule_in(r1)),
            ("schedulePeriodically", schedule_periodically(r2))
        ])
    }
}
