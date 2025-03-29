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

use std::ffi::{c_char, c_double, c_int, c_void};

/* mark for precompiled code (`<esc>Lua') */
pub const SIGNATURE: &[u8] = b"\033Lua";

pub const REGISTRYINDEX: c_int = -10000;
pub const ENVIRONINDEX: c_int = -10001;
pub const GLOBALSINDEX: c_int = -10002;

pub const fn upvalueindex(i: c_int) -> c_int {
    GLOBALSINDEX - i
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadStatus {
    Ok = 0,
    Yield = 1,
    ErrRun = 2,
    ErrSyntax = 3,
    ErrMem = 4,
    ErrErr = 5
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct State(*mut c_void);

pub type CFunction = extern "C-unwind" fn(l: State) -> i32;

pub type Reader = extern "C" fn(l: State, ud: *mut c_void, sz: usize) -> *const c_char;

pub type Writer = extern "C" fn(l: State, p: *const c_void, sz: usize, ud: *mut c_void) -> c_int;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    None = -1,
    Nil = 0,
    Boolean = 1,
    LightUserdata = 2,
    Number = 3,
    String = 4,
    Table = 5,
    Function = 6,
    Userdata = 7,
    Thread = 8
}

pub type Number = c_double;
pub type Integer = isize;


//--------------------
// State manipulation
//--------------------
extern "C" {
    pub fn lua_close(l: State);

    pub fn lua_atpanic(l: State, panicf: CFunction) -> CFunction;

    pub fn lua_newthread(l: State) -> State;
}

//--------------------------
// Basic stack manipulation
//--------------------------
extern "C" {
    pub fn lua_gettop(l: State) -> c_int;
    pub fn lua_settop(l: State, idx: c_int);
    pub fn lua_pushvalue(l: State, idx: c_int);
    pub fn lua_remove(l: State, idx: c_int);
    pub fn lua_insert(l: State, idx: c_int);
    pub fn lua_replace(l: State, idx: c_int);
    pub fn lua_checkstack(l: State, sz: c_int) -> c_int;

    pub fn lua_xmove(from: State, to: State, n: c_int);
}

//-------------------------------
// Access functions (stack -> C)
//-------------------------------
extern "C" {
    pub fn lua_isnumber(l: State, idx: c_int) -> c_int;
    pub fn lua_isstring(l: State, idx: c_int) -> c_int;
    pub fn lua_iscfunction(l: State, idx: c_int) -> c_int;
    pub fn lua_isuserdata(l: State, idx: c_int) -> c_int;
    pub fn lua_type(l: State, idx: c_int) -> Type;
    pub fn lua_typename(l: State, tp: c_int) -> *const c_char;

    pub fn lua_equal(l: State, idx1: c_int, idx2: c_int) -> c_int;
    pub fn lua_rawequal(l: State, idx1: c_int, idx2: c_int) -> c_int;
    pub fn lua_lessthan(l: State, idx1: c_int, idx2: c_int) -> c_int;

    pub fn lua_tonumber(l: State, idx: c_int) -> Number;
    pub fn lua_tointeger(l: State, idx: c_int) -> Integer;
    pub fn lua_toboolean(l: State, idx: c_int) -> c_int;
    pub fn lua_tolstring(l: State, idx: c_int, len: *mut usize) -> *const c_char;
    pub fn lua_objlen(l: State, idx: c_int) -> usize;
    pub fn lua_tocfunction(l: State, idx: c_int) -> CFunction;
    pub fn lua_touserdata(l: State, idx: c_int) -> *mut c_void;
    pub fn lua_tothread(l: State, idx: c_int) -> State;
    pub fn lua_topointer(l: State, idx: c_int) -> *const c_void;
}

//-------------------------------
// Push functions (C -> stack)
//-------------------------------
extern "C" {
    pub fn lua_pushnil(l: State);
    pub fn lua_pushnumber(l: State, n: Number);
    pub fn lua_pushinteger(l: State, n: Integer);
    pub fn lua_pushlstring(l: State, s: *const c_char, len: usize);
    pub fn lua_pushstring(l: State, s: *const c_char);
    //LUA_API const char *(lua_pushvfstring) (lua_State *L, const char *fmt, va_list argp);
    pub fn lua_pushfstring(l: State, fmt: *const c_char, ...) -> *const c_char;
    pub fn lua_pushcclosure(l: State, fun: CFunction, n: c_int);
    pub fn lua_pushboolean(l: State, b: c_int);
    pub fn lua_pushlightuserdata(l: State, p: *mut c_void);
    pub fn lua_pushthread(l: State);
}

//-------------------------------
// Get functions (Lua -> stack)
//-------------------------------
extern "C" {
    pub fn lua_gettable(l: State,  idx: c_int);
    pub fn lua_getfield(l: State,  idx: c_int, k: *const c_char);
    pub fn lua_rawget(l: State,  idx: c_int);
    pub fn lua_rawgeti(l: State,  idx: c_int, n: c_int);
    pub fn lua_createtable(l: State,  narr: c_int, nrec: c_int);
    pub fn lua_newuserdata(l: State, sz: usize) -> *mut c_void;
    pub fn lua_getmetatable(l: State, objindex: c_int) -> c_int;
    pub fn lua_getfenv(l: State, idx: c_int);
}

//-------------------------------
// Set functions (stack -> Lua)
//-------------------------------
extern "C" {
    pub fn lua_settable(l: State, idx: c_int);
    pub fn lua_setfield(l: State, idx: c_int, k: *const c_char);
    pub fn lua_rawset(l: State, idx: c_int);
    pub fn lua_rawseti(l: State, idx: c_int, n: c_int);
    pub fn lua_setmetatable(l: State, objindex: c_int) -> c_int;
    pub fn lua_setfenv(l: State, idx: c_int) -> c_int;
}

//-----------------------------------------------------
// `load' and `call' functions (load and run Lua code)
//-----------------------------------------------------
extern "C" {
    pub fn lua_call(l: State, nargs: c_int, nresults: c_int);
    pub fn lua_pcall(l: State, nargs: c_int, nresults: c_int, errfunc: c_int) -> ThreadStatus;
    pub fn lua_cpcall(l: State, func: CFunction, ud: *mut c_void) -> c_int;
    pub fn lua_load(l: State, reader: Reader, dt: *mut c_void, chunkname: *const c_char) -> ThreadStatus;

    pub fn lua_dump(l: State, writer: Writer, data: *mut c_void) -> c_int;
}

//---------------------
// Coroutine functions
//---------------------
extern "C" {
    pub fn lua_yield(l: State, nresults: c_int) -> c_int;
    pub fn lua_resume(l: State, narg: c_int) -> ThreadStatus;
    pub fn lua_status(l: State) -> ThreadStatus;
}

//-----------------------------------------
// Garbage-collection function and options
//-----------------------------------------
#[repr(i32)]
pub enum Gc {
    Stop = 0,
    Restart = 1,
    Collect = 2,
    Count = 3,
    Countb = 4,
    Step = 5,
    SetPause = 6,
    SetStepMul = 7,
    IsRunning = 9,
}

extern "C" {
    pub fn lua_gc(l: State, what: Gc, data: c_int) -> c_int;
}

//-------------------------
// Miscellaneous functions
//-------------------------
extern "C" {
    pub fn lua_error(l: State) -> c_int;
    pub fn lua_next(l: State, idx: c_int) -> c_int;
    pub fn lua_concat(l: State, n: c_int);
}
