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

use crate::ffi::ext::lua_ext_getprovenance;
use crate::vm::registry::core::RawKey;
use crate::vm::registry::{FromIndex, Set};
use crate::vm::Vm;

pub struct VmCheckedRawKey {
    raw: RawKey,
    provenance: u64,
}

impl VmCheckedRawKey {
    pub fn push(&self, vm: &Vm) {
        assert_eq!(
            unsafe { lua_ext_getprovenance(vm.as_ptr()) },
            self.provenance
        );
        unsafe { self.raw.push(vm) }
    }

    pub fn delete(self, vm: &Vm) {
        assert_eq!(
            unsafe { lua_ext_getprovenance(vm.as_ptr()) },
            self.provenance
        );
        unsafe { self.raw.delete(vm) }
    }

    #[inline(always)]
    pub fn as_raw(&self) -> RawKey {
        self.raw
    }
}

impl FromIndex for VmCheckedRawKey {
    unsafe fn from_index(vm: &Vm, index: i32) -> Self {
        let raw = RawKey::from_index(vm, index);
        Self {
            provenance: unsafe { lua_ext_getprovenance(vm.as_ptr()) },
            raw,
        }
    }
}

impl Set for VmCheckedRawKey {
    unsafe fn set(&self, vm: &Vm, index: i32) {
        assert_eq!(
            unsafe { lua_ext_getprovenance(vm.as_ptr()) },
            self.provenance
        );
        self.raw.set(vm, index);
    }
}
