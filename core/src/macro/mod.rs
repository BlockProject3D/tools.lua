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

mod lib_func;
mod userdata_func;
mod userdata;
mod closure;

#[macro_export]
macro_rules! c_stringify {
    ($str: ident) => {
        unsafe { std::ffi::CStr::from_ptr(concat!(stringify!($str), "\0").as_ptr() as _) }
    };
}

#[macro_export]
macro_rules! decl_from_param {
    (
        $vm: ident, $start_index: literal, 
    ) => {
    };

    (
        $vm: ident, $start_index: literal, $arg_name: ident: $arg_ty: ty
    ) => {
        use $crate::vm::function::FromParam;
        let $arg_name: $arg_ty = unsafe { FromParam::from_param($vm, $start_index) };
    };

    (
        $vm: ident, $start_index: literal, $($arg_name: ident: $arg_ty: ty)*
    ) => {
        use $crate::vm::function::FromParam;
        let mut index = $start_index;
        $crate::decl_from_param!(_from_param $vm, index, $(($arg_name: $arg_ty))*);
    };

    (_from_param $vm: ident, $index: ident, ) => { };

    (_from_param $vm: ident, $index: ident, ($arg_name: ident: $arg_ty: ty)) => {
        let $arg_name: $arg_ty = unsafe { FromParam::from_param($vm, $index) };
    };

    (_from_param $vm: ident, $index: ident, ($arg_name: ident: $arg_ty: ty) $(($arg_name2: ident: $arg_ty2: ty))*) => {
        let $arg_name: $arg_ty = unsafe { FromParam::from_param($vm, $index) };
        $index += 1;
        $crate::decl_from_param!(_from_param $vm, $index, $(($arg_name2: $arg_ty2))*);
    };
}
