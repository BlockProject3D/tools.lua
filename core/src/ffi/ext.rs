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

use crate::ffi::lua::{RawInteger, RawNumber, State};
use std::ffi::{c_int, c_void};

pub type MSize = u32;

//---------------
// Value reading
//---------------
extern "C" {
    pub fn lua_ext_fast_checknumber(l: State, numarg: c_int) -> RawNumber;
    pub fn lua_ext_fast_checkinteger(l: State, numarg: c_int) -> RawInteger;
    pub fn lua_ext_fast_checkboolean(l: State, numarg: c_int) -> c_int;
}

//-----------------
// 64 bit integers
//-----------------
extern "C" {
    pub fn lua_ext_checkinteger64(l: State, numarg: c_int) -> i64;
    pub fn lua_ext_checkuinteger64(l: State, numarg: c_int) -> u64;
    pub fn lua_ext_getinteger64(l: State, numarg: c_int, out: *mut i64) -> c_int;
    pub fn lua_ext_getuinteger64(l: State, numarg: c_int, out: *mut u64) -> c_int;
    pub fn lua_ext_pushinteger64(l: State, value: i64) -> c_int;
    pub fn lua_ext_pushuinteger64(l: State, value: u64) -> c_int;
    pub fn lua_ext_tointeger64(l: State, numarg: c_int) -> i64;
    pub fn lua_ext_touinteger64(l: State, numarg: c_int) -> u64;
}

//-------
// Other
//-------
extern "C" {
    pub fn lua_ext_tab_len(l: State, idx: c_int, outsize: *mut MSize) -> c_int;
    pub fn lua_ext_ccatch_error(l: State) -> u32;
}

//-----
// JIT
//-----
extern "C" {
    /// Sets the global mode of the JIT.
    pub fn lua_ext_setjitmode(l: State, mode: c_int) -> c_int;

    /// Returns global flags of the JIT.
    pub fn lua_ext_getjitflags(l: State) -> u32;

    /// Sets global JIT flags.
    pub fn lua_ext_setjitflags(l: State, flags: u32) -> c_int;
}

//---------------------
// Named registry keys
//---------------------
extern "C" {
    pub fn lua_ext_keyreg_get() -> *mut c_void;
    pub fn lua_ext_keyreg_ref(ptr: *mut c_void) -> *mut c_void;
    pub fn lua_ext_keyreg_unref() -> *mut c_void;
}
