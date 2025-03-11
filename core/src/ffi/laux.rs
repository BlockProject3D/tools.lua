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

use std::ffi::{c_char, c_int, c_void};
use crate::ffi::lua::{Integer, Number, State, ThreadStatus, Type};

//--------------------
// State manipulation
//--------------------
extern "C" {
    pub fn luaL_newstate() -> State;
    pub fn luaL_openlibs(l: State);
}

//----------------
// Error handling
//----------------
extern "C" {
    pub fn luaL_error(l: State, fmt: *const c_char, ...) -> c_int;
    pub fn luaL_typerror(l: State, narg: c_int, tname: *const c_char) -> c_int;
    pub fn luaL_argerror(l: State, numarg: c_int, extramsg: *const c_char) -> c_int;

    pub fn luaL_traceback(l: State, l1: State, msg: *const c_char, level: c_int);
}

//---------------
// Value reading
//---------------
extern "C" {
    pub fn luaL_checklstring(l: State, numarg: c_int, len: *mut usize) -> *const c_char;
    pub fn luaL_optlstring(l: State, numarg: c_int, def: *const c_char, len: *mut usize) -> *const c_char;

    pub fn luaL_checknumber(l: State, numarg: c_int) -> Number;
    pub fn luaL_optnumber(l: State, narg: c_int, def: Number) -> Number;

    pub fn luaL_checkinteger(l: State, numarg: c_int) -> Integer;
    pub fn luaL_optinteger(l: State, narg: c_int, def: Integer) -> Integer;

    pub fn luaL_checkstack(l: State, sz: c_int, msg: *const c_char);
    pub fn luaL_checktype(l: State, narg: c_int, t: Type);
    pub fn luaL_checkany(l: State, narg: c_int);

    pub fn luaL_checkudata(l: State, ud: c_int, tname: *const c_char) -> *mut c_void;
    pub fn luaL_testudata(l: State, ud: c_int, tname: *const c_char) -> *mut c_void;
}

//------------------------
// Metatable manipulation
//------------------------
extern "C" {
    pub fn luaL_newmetatable(l: State, tname: *const c_char) -> c_int;
    pub fn luaL_setmetatable(l: State, tname: *const c_char);
    pub fn luaL_getmetafield(l: State, obj: c_int, e: *const c_char) -> c_int;
    pub fn luaL_callmeta(l: State, obj: c_int, e: *const c_char) -> c_int;
}

//-------------------------
// Miscellaneous functions
//-------------------------
extern "C" {
    pub fn luaL_where(l: State, lvl: c_int);

    pub fn luaL_checkoption(l: State, narg: c_int, def: *const c_char, lst: *const *const c_char) -> c_int;
}

//----------
// Registry
//----------
pub const NOREF: c_int = -2;
pub const REFNIL: c_int = -1;

extern "C" {
    pub fn luaL_ref(l: State, t: c_int) -> c_int;
    pub fn luaL_unref(l: State, t: c_int, r: c_int);
}

//------------------
// Loading lua code
//------------------
extern "C" {
    pub fn luaL_loadfile(l: State, filename: *const c_char) -> ThreadStatus;
    pub fn luaL_loadbuffer(l: State, buff: *const c_char, sz: usize, name: *const c_char) -> ThreadStatus;
    pub fn luaL_loadstring(l: State, s: *const c_char) -> ThreadStatus;
}
