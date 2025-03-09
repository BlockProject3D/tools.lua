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
macro_rules! decl_lib_func {
    (
        $vis: vis fn $fn_name: ident ($name: ident: &Vm$(, $($arg_name: ident: $arg_ty: ty),*)?) -> $ret_ty: ty $code: block
    ) => {
        $vis extern "C-unwind" fn $fn_name(l: $crate::ffi::lua::State) -> i32 {
            fn _func($name: &$crate::vm::Vm$(, $($arg_name: $arg_ty),*)?) -> $ret_ty $code
            use $crate::vm::function::FromParam;
            use $crate::vm::function::IntoParam;
            let vm = unsafe { $crate::vm::Vm::from_raw(l) };
            let mut index = 1;
            $($(
                let $arg_name: $arg_ty = unsafe { FromParam::from_param(&vm, index) };
                index += 1;
            )*)?
            let ret = _func(&vm $(, $($arg_name),*)?);
            ret.into_param(&vm) as _
        }
    };
    (
        $vis: vis fn $fn_name: ident ($($arg_name: ident: $arg_ty: ty),*) -> $ret_ty: ty $code: block
    ) => {
        $vis extern "C-unwind" fn $fn_name(l: $crate::ffi::lua::State) -> i32 {
            fn _func($($arg_name: $arg_ty),*) -> $ret_ty $code
            use $crate::vm::function::FromParam;
            use $crate::vm::function::IntoParam;
            let vm = unsafe { $crate::vm::Vm::from_raw(l) };
            let mut index = 1;
            $(
                let $arg_name: $arg_ty = unsafe { FromParam::from_param(&vm, index) };
                index += 1;
            )*
            let ret = _func($($arg_name),*);
            ret.into_param(&vm) as _
        }
    };
}
