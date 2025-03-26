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
macro_rules! decl_closure {
    (
        $vis: vis fn $fn_name: ident $(<$lifetime: lifetime>)? |$upvalue_name: ident: $upvalue_ty: ty| ($name: ident: &Vm$(, $($arg_name: ident: $arg_ty: ty),*)?) -> $ret_ty: ty $code: block
    ) => {
        $vis fn $fn_name(upvalue: <$upvalue_ty as $crate::vm::closure::FromUpvalue>::Into) -> $crate::vm::closure::types::RClosure<<$upvalue_ty as $crate::vm::closure::FromUpvalue>::Into> {
            extern "C-unwind" fn _cfunc(l: $crate::ffi::lua::State) -> i32 {
                fn _func($name: &$crate::vm::Vm, $upvalue_name: <$upvalue_ty as $crate::vm::closure::Upvalue>::From<'_>$(, $($arg_name: $arg_ty),*)?) -> $ret_ty $code
                use $crate::vm::function::IntoParam;
                let vm = unsafe { $crate::vm::Vm::from_raw(l) };
                #[inline(always)]
                extern "C-unwind" fn _vmfunc $(<$lifetime>)? (vm: &$($lifetime)? $crate::vm::Vm) -> i32 {
                    let $upvalue_name: <$upvalue_ty as $crate::vm::closure::Upvalue>::From<'_> = unsafe { $crate::vm::closure::FromUpvalue::from_upvalue(vm, 1) };
                    $($crate::decl_from_param!(vm, 1, $($arg_name: $arg_ty)*);)?
                    let ret = _func(vm, $upvalue_name $(, $($arg_name),*)?);
                    ret.into_param(vm) as _
                }
                _vmfunc(&vm)
            }
            $crate::vm::closure::types::RClosure::new(_cfunc, upvalue)
        }
    };
    (
        $vis: vis fn $fn_name: ident $(<$lifetime: lifetime>)? |$upvalue_name: ident: $upvalue_ty: ty|  ($($arg_name: ident: $arg_ty: ty),*) -> $ret_ty: ty $code: block
    ) => {
        $vis fn $fn_name(upvalue: $upvalue_ty) -> $crate::vm::closure::types::RClosure<$upvalue_ty> {
            extern "C-unwind" fn _cfunc(l: $crate::ffi::lua::State) -> i32 {
                fn _func<'a>($upvalue_name: <$upvalue_ty as $crate::vm::closure::Upvalue>::From<'_>, $($arg_name: $arg_ty),*) -> $ret_ty $code
                use $crate::vm::function::IntoParam;
                let vm = unsafe { $crate::vm::Vm::from_raw(l) };
                #[inline(always)]
                extern "C-unwind" fn _vmfunc $(<$lifetime>)? (vm: &$($lifetime)? $crate::vm::Vm) -> i32 {
                    let $upvalue_name: <$upvalue_ty as $crate::vm::closure::Upvalue>::From<'_> = unsafe { $crate::vm::closure::FromUpvalue::from_upvalue(vm, 1) };
                    $crate::decl_from_param!(vm, 1, $($arg_name: $arg_ty)*);
                    let ret = _func($upvalue_name, $($arg_name),*);
                    ret.into_param(vm) as _
                }
                _vmfunc(&vm)
            }
            $crate::vm::closure::types::RClosure::new(_cfunc, upvalue)
        }
    };
}
