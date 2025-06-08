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

pub const MODE_FLUSH: c_int = 0x0200;
pub const MODE_ON: c_int = 0x0100;
pub const MODE_OFF: c_int = 0x0000;

pub const F_ON: u32 = 0x00000001;

pub const F_CPU: u32 = 0x00000010;

#[cfg(target_arch = "x86_64")]
mod x86_64 {
    use crate::ffi::jit::F_CPU;

    pub const F_SSE3: u32 = F_CPU << 0;
    pub const F_SSE4_1: u32 = F_CPU << 1;
    pub const F_BMI2: u32 = F_CPU << 2;
}

#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
mod arm {
    use crate::ffi::jit::F_CPU;

    pub const F_ARMV6_: u32 = F_CPU << 0;
    pub const F_ARMV6T2_: u32 = F_CPU << 1;
    pub const F_ARMV7: u32 = F_CPU << 2;
    pub const F_ARMV8: u32 = F_CPU << 3;
    pub const F_VFPV2: u32 = F_CPU << 4;
    pub const F_VFPV3: u32 = F_CPU << 5;

    pub const F_ARMV6: u32 = F_ARMV6_|F_ARMV6T2_|F_ARMV7|F_ARMV8;
    pub const F_ARMV6T2: u32 = F_ARMV6T2_|F_ARMV7|F_ARMV8;
    pub const F_VFP: u32 = F_VFPV2|F_VFPV3;
}

#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
pub use arm::*;

#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

pub const F_OPT: u32 = 0x00010000;
pub const F_OPT_MASK: u32 = 0x0fff0000;

pub const F_OPT_FOLD: u32 = F_OPT << 0;
pub const F_OPT_CSE: u32 = F_OPT << 1;
pub const F_OPT_DCE: u32 = F_OPT << 2;
pub const F_OPT_FWD: u32 = F_OPT << 3;
pub const F_OPT_DSE: u32 = F_OPT << 4;
pub const F_OPT_NARROW: u32 = F_OPT << 5;
pub const F_OPT_LOOP: u32 = F_OPT << 6;
pub const F_OPT_ABC: u32 = F_OPT << 7;
pub const F_OPT_SINK: u32 = F_OPT << 8;
pub const F_OPT_FUSE: u32 = F_OPT << 9;
pub const F_OPT_FMA: u32 = F_OPT << 10;

pub const F_OPT_0: u32 = 0;
pub const F_OPT_1: u32 = F_OPT_FOLD | F_OPT_CSE | F_OPT_DCE;
pub const F_OPT_2: u32 = F_OPT_1 | F_OPT_NARROW | F_OPT_LOOP;
pub const F_OPT_3: u32 = F_OPT_2 | F_OPT_FWD | F_OPT_DSE | F_OPT_ABC | F_OPT_SINK | F_OPT_FUSE;
pub const F_OPT_DEFAULT: u32 = F_OPT_3;
