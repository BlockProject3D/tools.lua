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

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use bp3d_debug::{error, warning};
use bp3d_os::time::Instant;
use bp3d_lua::util::LuaThread;
use bp3d_lua::vm::thread::core::State;
use bp3d_lua::vm::thread::value::Value;
use bp3d_lua::vm::Vm;
use crate::data::DataOut;
use crate::data_out::Log;

struct Task {
    at_ms: u64,
    period_ms: Option<u32>,
    thread: LuaThread
}

impl Eq for Task {}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.at_ms.cmp(&other.at_ms)
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.thread.as_thread() == other.thread.as_thread()
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.at_ms.partial_cmp(&self.at_ms)
    }
}

struct Scheduler {
    main: BinaryHeap<Task>,
    instant: Instant
}

impl Scheduler {
    pub fn schedule_in(&mut self, value: Value, after_ms: u32) {
        let task = Task {
            at_ms: self.instant.elapsed().as_millis() as u64 + after_ms as u64,
            period_ms: None,
            thread: LuaThread::create(value)
        };
        self.main.push(task);
    }

    pub fn schedule_periodically(&mut self, value: Value, period_ms: u32) {
        let task = Task {
            at_ms: self.instant.elapsed().as_millis() as u64 + period_ms as u64,
            period_ms: Some(period_ms),
            thread: LuaThread::create(value)
        };
        self.main.push(task);
    }

    pub fn step(&mut self, vm: &Vm, logger: &DataOut) {
        let time = self.instant.elapsed().as_millis() as u64;
        while let Some(task) = self.main.peek() {
            if time >= task.at_ms {
                let mut task = unsafe { self.main.pop().unwrap_unchecked() };
                let out = match task.thread.as_thread().resume::<Option<u32>>(()) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("{}: Failed to schedule lua thread: {:?}", task.thread.as_thread(), e);
                        logger.send(Log("scheduler", e.to_string()));
                        task.thread.delete(vm);
                        continue;
                    }
                };
                if let Some(new_time_ms) = out.data {
                    if out.state == State::Suspended {
                        if task.period_ms.is_some() {
                            task.period_ms = Some(new_time_ms);
                        }
                        task.at_ms = time + new_time_ms as u64;
                        self.main.push(task);
                        continue;
                    } else {
                        warning!("{}: Attempt to change period or time of terminated task.", task.thread.as_thread());
                        task.thread.delete(vm);
                        continue;
                    }
                }
                if let Some(period_ms) = task.period_ms {
                    if out.state == State::Suspended {
                        task.at_ms = time + period_ms as u64;
                        self.main.push(task);
                    } else {
                        warning!("{}: Attempt to re-schedule terminated task.", task.thread.as_thread());
                        task.thread.delete(vm);
                    }
                }
            } else {
                break;
            }
        }
    }
}

pub struct SchedulerPtr(RefCell<Scheduler>);

impl SchedulerPtr {
    pub fn new() -> Self {
        Self(RefCell::new(Scheduler { main: BinaryHeap::new(), instant: Instant::now() }))
    }

    pub fn schedule_in(&self, value: Value, after_ms: u32) {
        let mut inner = self.0.borrow_mut();
        inner.schedule_in(value, after_ms);
    }

    pub fn schedule_periodically(&self, value: Value, period_ms: u32) {
        let mut inner = self.0.borrow_mut();
        inner.schedule_periodically(value, period_ms);
    }

    pub fn step(&self, vm: &Vm, logger: &DataOut) {
        let mut inner = self.0.borrow_mut();
        inner.step(vm, logger);
    }
}
