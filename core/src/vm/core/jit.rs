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

use std::ffi::c_int;
use std::fmt::{Display, Formatter};
use crate::ffi::ext::{lua_ext_getjitflags, lua_ext_setjitflags, lua_ext_setjitmode};
use crate::ffi::jit;
use crate::vm::{RootVm, Vm};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CpuArm {
    pub v6: bool,
    pub v6t2: bool,
    pub v7: bool,
    pub v8: bool,
    pub vfpv2: bool,
    pub vfpv3: bool,
}

impl CpuArm {
    pub fn has_vfp(&self) -> bool {
        self.vfpv2 || self.vfpv3
    }
}

impl Display for CpuArm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if !self.v6 && !self.v7 && !self.v8 && !self.v6t2 && !self.vfpv2 && !self.vfpv3 {
            write!(f, "ARM CPU, no features")?;
        } else {
            write!(f, "ARM CPU, features:")?;
            if self.v6 {
                write!(f, " ARMV6")?;
            }
            if self.v6t2 {
                write!(f, " ARMV6T2")?;
            }
            if self.v7 {
                write!(f, " ARMV7")?;
            }
            if self.v8 {
                write!(f, " ARMV8")?;
            }
            if self.vfpv2 {
                write!(f, " VFPV2")?;
            }
            if self.vfpv3 {
                write!(f, " VFPV3")?;
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CpuX86 {
    pub sse3: bool,
    pub sse4_1: bool,
    pub bmi2: bool,
}

impl Display for CpuX86 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if !self.sse3 && !self.sse4_1 && !self.bmi2 {
            write!(f, "X86 CPU, no features")?;
        } else {
            write!(f, "X86 CPU, features:")?;
            if self.sse3 {
                write!(f, " SSE3")?;
            }
            if self.sse4_1 {
                write!(f, " SSE4_1")?;
            }
            if self.bmi2 {
                write!(f, " BMI2")?;
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Cpu {
    X86(CpuX86),
    Arm(CpuArm),
}

impl Display for Cpu {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Cpu::X86(cpu) => cpu.fmt(f),
            Cpu::Arm(cpu) => cpu.fmt(f),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Opts {
    pub fold: bool,
    pub cse: bool,
    pub dce: bool,
    pub fwd: bool,
    pub dse: bool,
    pub narrow: bool,
    pub loop1: bool,
    pub abc: bool,
    pub sink: bool,
    pub fuse: bool,
    pub fma: bool
}

impl Display for Opts {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "optimizations:")?;
        if self.fold {
            write!(f, " fold")?;
        }
        if self.cse {
            write!(f, " cse")?;
        }
        if self.dce {
            write!(f, " dce")?;
        }
        if self.fwd {
            write!(f, " fwd")?;
        }
        if self.dse {
            write!(f, " dse")?;
        }
        if self.narrow {
            write!(f, " narrow")?;
        }
        if self.loop1 {
            write!(f, " loop")?;
        }
        if self.abc {
            write!(f, " abc")?;
        }
        if self.sink {
            write!(f, " sink")?;
        }
        if self.fuse {
            write!(f, " fuse")?;
        }
        if self.fma {
            write!(f, " fma")?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub enum OptLevel {
    O0,
    O1,
    O2,
    #[default]
    O3,
    Unknown
}

impl Display for OptLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OptLevel::O0 => write!(f, "O0"),
            OptLevel::O1 => write!(f, "O1"),
            OptLevel::O2 => write!(f, "O2"),
            OptLevel::O3 => write!(f, "O3"),
            OptLevel::Unknown => write!(f, "Unknown")
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct JitOptions {
    cur_flag_set: u32,
    mode: c_int,
    opt_level_changed: bool
}

impl JitOptions {
    /// Read JIT options for the given [Vm](Vm).
    ///
    /// # Arguments
    ///
    /// * `vm`: the [Vm] instance to read options for.
    ///
    /// returns: JitOptions
    pub fn get(vm: &Vm) -> JitOptions {
        Self {
            cur_flag_set: unsafe { lua_ext_getjitflags(vm.as_ptr()) },
            mode: -1,
            opt_level_changed: false,
        }
    }

    pub fn is_enabled(&self) -> bool {
        (self.cur_flag_set & jit::F_ON) != 0
    }

    pub fn cpu(&self) -> Cpu {
        #[cfg(target_arch = "x86_64")]
        let cpu = Cpu::X86(CpuX86 {
            sse3: (self.cur_flag_set & jit::F_SSE3) != 0,
            sse4_1: (self.cur_flag_set & jit::F_SSE4_1) != 0,
            bmi2: (self.cur_flag_set & jit::F_BMI2) != 0,
        });
        #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
        let cpu = Cpu::Arm(CpuArm {
            v6: (self.cur_flag_set & jit::F_ARMV6_) != 0,
            v6t2: (self.cur_flag_set & jit::F_ARMV6T2_) != 0,
            v7: (self.cur_flag_set & jit::F_ARMV7) != 0,
            v8: (self.cur_flag_set & jit::F_ARMV8) != 0,
            vfpv2: (self.cur_flag_set & jit::F_VFPV2) != 0,
            vfpv3: (self.cur_flag_set & jit::F_VFPV3) != 0,
        });
        cpu
    }

    pub fn opts(&self) -> Opts {
        Opts {
            fold: (self.cur_flag_set & jit::F_OPT_FOLD) != 0,
            cse: (self.cur_flag_set & jit::F_OPT_CSE) != 0,
            dce: (self.cur_flag_set & jit::F_OPT_DCE) != 0,
            fwd: (self.cur_flag_set & jit::F_OPT_FWD) != 0,
            dse: (self.cur_flag_set & jit::F_OPT_DSE) != 0,
            narrow: (self.cur_flag_set & jit::F_OPT_NARROW) != 0,
            loop1: (self.cur_flag_set & jit::F_OPT_LOOP) != 0,
            abc: (self.cur_flag_set & jit::F_OPT_ABC) != 0,
            sink: (self.cur_flag_set & jit::F_OPT_SINK) != 0,
            fuse: (self.cur_flag_set & jit::F_OPT_FUSE) != 0,
            fma: (self.cur_flag_set & jit::F_OPT_FMA) != 0,
        }
    }

    pub fn opt_level(&self) -> OptLevel {
        if (self.cur_flag_set & jit::F_OPT_3) == jit::F_OPT_3 {
            return OptLevel::O3;
        }
        if (self.cur_flag_set & jit::F_OPT_2) == jit::F_OPT_2 {
            return OptLevel::O2;
        }
        if (self.cur_flag_set & jit::F_OPT_1) == jit::F_OPT_1 {
            return OptLevel::O1;
        }
        if (self.cur_flag_set & jit::F_OPT_MASK) == jit::F_OPT_0 {
            return OptLevel::O0;
        }
        OptLevel::Unknown
    }

    pub fn flush(&mut self) {
        self.mode = jit::MODE_FLUSH;
    }

    pub fn disable(&mut self) {
        if self.is_enabled() {
            self.mode = jit::MODE_OFF;
        }
    }

    pub fn enable(&mut self) {
        if !self.is_enabled() {
            self.mode = jit::MODE_ON;
        }
    }

    pub fn set_opt_level(&mut self, level: OptLevel) {
        self.cur_flag_set &= !jit::F_OPT_MASK;
        match level {
            OptLevel::O0 | OptLevel::Unknown => self.cur_flag_set |= jit::F_OPT_0,
            OptLevel::O1 => self.cur_flag_set |= jit::F_OPT_1,
            OptLevel::O2 => self.cur_flag_set |= jit::F_OPT_2,
            OptLevel::O3 => self.cur_flag_set |= jit::F_OPT_3
        }
        self.opt_level_changed = true;
    }

    pub fn apply(self, vm: &mut RootVm) {
        if self.mode != -1 {
            assert_eq!(unsafe { lua_ext_setjitmode(vm.as_ptr(), self.mode) }, 0);
        }
        if self.opt_level_changed {
            assert_eq!(unsafe { lua_ext_setjitflags(vm.as_ptr(), self.cur_flag_set) }, 0);
        }
    }
}
