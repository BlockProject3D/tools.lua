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

#[macro_export]
macro_rules! decl_userdata_func {
    (
        $vis: vis fn $fn_name: ident $(<$lifetime: lifetime>)? ($this: ident: &mut $obj_name: ident, $name: ident: &Vm$(, $($arg_name: ident: $arg_ty: ty),*)?) -> $ret_ty: ty $code: block
    ) => {
        impl $obj_name {
            $vis fn $fn_name() -> $crate::vm::userdata::core::Function {
                extern "C-unwind" fn _cfunc(l: $crate::ffi::lua::State) -> i32 {
                    fn _func $(<$lifetime>)? ($this: &mut $obj_name, $name: &$crate::vm::Vm$(, $($arg_name: $arg_ty),*)?) -> $ret_ty $code
                    use $crate::vm::function::IntoParam;
                    let this_ptr = unsafe { $crate::ffi::laux::luaL_checkudata(l, 1, <$obj_name as $crate::vm::userdata::UserData>::CLASS_NAME.as_ptr()) } as *mut $obj_name;
                    let vm = unsafe { $crate::vm::Vm::from_raw(l) };
                    #[inline(always)]
                    extern "C-unwind" fn _vmfunc $(<$lifetime>)? (this_ptr: *mut $obj_name, vm: &$($lifetime)? $crate::vm::Vm) -> i32 {
                        $($crate::decl_from_param!(vm, 2, $($arg_name: $arg_ty)*);)?
                        let ret = _func(unsafe { &mut *this_ptr }, vm $(, $($arg_name),*)?);
                        ret.into_param(vm) as _
                    }
                    _vmfunc(this_ptr, &vm)
                }
                let mut f = $crate::vm::userdata::core::Function::new($crate::c_stringify!($fn_name), _cfunc);
                f.mutable();
                f.arg::<&$obj_name>();
                $($(f.arg::<$arg_ty>();)*)?
                f
            }
        }
    };
    (
        $vis: vis fn $fn_name: ident $(<$lifetime: lifetime>)? ($this: ident: &mut $obj_name: ident$(, $($arg_name: ident: $arg_ty: ty),*)?) -> $ret_ty: ty $code: block
    ) => {
        impl $obj_name {
            $vis fn $fn_name() -> $crate::vm::userdata::core::Function {
                extern "C-unwind" fn _cfunc(l: $crate::ffi::lua::State) -> i32 {
                    fn _func $(<$lifetime>)? ($this: &mut $obj_name$(, $($arg_name: $arg_ty),*)?) -> $ret_ty $code
                    use $crate::vm::function::IntoParam;
                    let this_ptr = unsafe { $crate::ffi::laux::luaL_checkudata(l, 1, <$obj_name as $crate::vm::userdata::UserData>::CLASS_NAME.as_ptr()) } as *mut $obj_name;
                    let vm = unsafe { $crate::vm::Vm::from_raw(l) };
                    #[inline(always)]
                    extern "C-unwind" fn _vmfunc $(<$lifetime>)? (this_ptr: *mut $obj_name, vm: &$($lifetime)? $crate::vm::Vm) -> i32 {
                        $($crate::decl_from_param!(vm, 2, $($arg_name: $arg_ty)*);)?
                        let ret = _func(unsafe { &mut *this_ptr } $(, $($arg_name),*)?);
                        ret.into_param(vm) as _
                    }
                    _vmfunc(this_ptr, &vm)
                }
                let mut f = $crate::vm::userdata::core::Function::new($crate::c_stringify!($fn_name), _cfunc);
                f.mutable();
                f.arg::<&$obj_name>();
                $($(f.arg::<$arg_ty>();)*)?
                f
            }
        }
    };
    (
        $vis: vis fn $fn_name: ident $(<$lifetime: lifetime>)? ($this: ident: &$obj_name: ident, $name: ident: &Vm$(, $($arg_name: ident: $arg_ty: ty),*)?) -> $ret_ty: ty $code: block
    ) => {
        impl $obj_name {
            $vis fn $fn_name() -> $crate::vm::userdata::core::Function {
                extern "C-unwind" fn _cfunc(l: $crate::ffi::lua::State) -> i32 {
                    fn _func $(<$lifetime>)? ($this: &$obj_name, $name: &$crate::vm::Vm$(, $($arg_name: $arg_ty),*)?) -> $ret_ty $code
                    use $crate::vm::function::IntoParam;
                    let this_ptr = unsafe { $crate::ffi::laux::luaL_checkudata(l, 1, <$obj_name as $crate::vm::userdata::UserData>::CLASS_NAME.as_ptr()) } as *const $obj_name;
                    let vm = unsafe { $crate::vm::Vm::from_raw(l) };
                    #[inline(always)]
                    extern "C-unwind" fn _vmfunc $(<$lifetime>)? (this_ptr: *const $obj_name, vm: &$($lifetime)? $crate::vm::Vm) -> i32 {
                        $($crate::decl_from_param!(vm, 2, $($arg_name: $arg_ty)*);)?
                        let ret = _func(unsafe { &*this_ptr }, vm $(, $($arg_name),*)?);
                        ret.into_param(vm) as _
                    }
                    _vmfunc(this_ptr, &vm)
                }
                let mut f = $crate::vm::userdata::core::Function::new($crate::c_stringify!($fn_name), _cfunc);
                f.arg::<&$obj_name>();
                $($(f.arg::<$arg_ty>();)*)?
                f
            }
        }
    };
    (
        $vis: vis fn $fn_name: ident $(<$lifetime: lifetime>)? ($this: ident: &$obj_name: ident$(, $($arg_name: ident: $arg_ty: ty),*)?) -> $ret_ty: ty $code: block
    ) => {
        impl $obj_name {
            $vis fn $fn_name() -> $crate::vm::userdata::core::Function {
                extern "C-unwind" fn _cfunc(l: $crate::ffi::lua::State) -> i32 {
                    fn _func $(<$lifetime>)? ($this: &$obj_name$(, $($arg_name: $arg_ty),*)?) -> $ret_ty $code
                    use $crate::vm::function::IntoParam;
                    let this_ptr = unsafe { $crate::ffi::laux::luaL_checkudata(l, 1, <$obj_name as $crate::vm::userdata::UserData>::CLASS_NAME.as_ptr()) } as *const $obj_name;
                    let vm = unsafe { $crate::vm::Vm::from_raw(l) };
                    #[inline(always)]
                    extern "C-unwind" fn _vmfunc $(<$lifetime>)? (this_ptr: *const $obj_name, vm: &$($lifetime)? $crate::vm::Vm) -> i32 {
                        $($crate::decl_from_param!(vm, 2, $($arg_name: $arg_ty)*);)?
                        let ret = _func(unsafe { &*this_ptr } $(, $($arg_name),*)?);
                        ret.into_param(vm) as _
                    }
                    _vmfunc(this_ptr, &vm)
                }
                let mut f = $crate::vm::userdata::core::Function::new($crate::c_stringify!($fn_name), _cfunc);
                f.arg::<&$obj_name>();
                $($(f.arg::<$arg_ty>();)*)?
                f
            }
        }
    };
}
