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

use crate::ffi::ext::lua_ext_fast_checkinteger;
use crate::ffi::lua::{lua_pushinteger, lua_tointeger, RawInteger, Type};
use crate::util::core::{SimpleDrop, TryFromIntError};
use crate::vm::function::{FromParam, IntoParam};
use crate::vm::util::{LuaType, TypeName};
use crate::vm::value::{FromLua, ImmutableValue, IntoLua};
use crate::vm::value::util::check_type_equals;
use crate::vm::Vm;

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Debug)]
pub struct Int53(i64);

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Debug)]
pub struct UInt53(u64);

impl Int53 {
    pub const MIN: Int53 = Int53(-(2 << 51));
    pub const MAX: Int53 = Int53((2 << 51) - 1);

    #[inline(always)]
    pub const fn to_i64(self) -> i64 {
        self.0
    }
}

impl UInt53 {
    pub const MIN: UInt53 = UInt53(0);
    pub const MAX: UInt53 = UInt53((2 << 52) - 1);

    #[inline(always)]
    pub const fn to_u64(self) -> u64 {
        self.0
    }
}

impl From<Int53> for i64 {
    #[inline(always)]
    fn from(val: Int53) -> i64 {
        val.to_i64()
    }
}

impl From<UInt53> for u64 {
    #[inline(always)]
    fn from(val: UInt53) -> u64 {
        val.to_u64()
    }
}

impl TryFrom<Int53> for UInt53 {
    type Error = TryFromIntError;

    fn try_from(value: Int53) -> Result<Self, Self::Error> {
        if value.0 < 0 {
            Err(TryFromIntError)
        } else {
            Ok(UInt53(value.0 as u64))
        }
    }
}

impl TryFrom<UInt53> for Int53 {
    type Error = TryFromIntError;

    fn try_from(value: UInt53) -> Result<Self, Self::Error> {
        if value.0 > Int53::MAX.0 as _ {
            Err(TryFromIntError)
        } else {
            Ok(Int53(value.0 as i64))
        }
    }
}

impl TryFrom<i64> for Int53 {
    type Error = TryFromIntError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value > Int53::MAX.0 || value < Int53::MIN.0 {
            Err(TryFromIntError)
        } else {
            Ok(Int53(value))
        }
    }
}

impl TryFrom<u64> for Int53 {
    type Error = TryFromIntError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > Int53::MAX.0 as _ {
            Err(TryFromIntError)
        } else {
            Ok(Int53(value as _))
        }
    }
}

impl TryFrom<i64> for UInt53 {
    type Error = TryFromIntError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value < 0 || value > UInt53::MAX.0 as _ {
            Err(TryFromIntError)
        } else {
            Ok(UInt53(value as _))
        }
    }
}

impl TryFrom<u64> for UInt53 {
    type Error = TryFromIntError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > UInt53::MAX.0 as _ {
            Err(TryFromIntError)
        } else {
            Ok(UInt53(value as _))
        }
    }
}

macro_rules! impl_from_lua {
    ($t: ty, $expected: ident, $func: ident, $push_func: ident) => {
        unsafe impl SimpleDrop for $t {}

        impl LuaType for $t {
            fn lua_type() -> Vec<TypeName> {
                vec![TypeName::Some(std::any::type_name::<RawInteger>())]
            }
        }

        impl FromLua<'_> for $t {
            #[inline(always)]
            unsafe fn from_lua_unchecked(vm: &Vm, index: i32) -> Self {
                Self($func(vm.as_ptr(), index) as _)
            }

            fn from_lua(vm: &Vm, index: i32) -> crate::vm::Result<Self> {
                check_type_equals(vm, index, Type::$expected)?;
                Ok(Self(unsafe { $func(vm.as_ptr(), index) as _ }))
            }
        }

        unsafe impl IntoLua for $t {
            #[inline(always)]
            fn into_lua(self, vm: &Vm) -> u16 {
                unsafe {
                    $push_func(vm.as_ptr(), self.0 as _);
                    1
                }
            }
        }

        unsafe impl ImmutableValue for $t {}

        impl FromParam<'_> for $t {
            #[inline(always)]
            unsafe fn from_param(vm: &Vm, index: i32) -> Self {
                Self(lua_ext_fast_checkinteger(vm.as_ptr(), index) as _)
            }

            #[inline(always)]
            fn try_from_param(vm: &Vm, index: i32) -> Option<Self> {
                FromLua::from_lua(vm, index).ok()
            }
        }

        unsafe impl IntoParam for $t {
            #[inline(always)]
            fn into_param(self, vm: &Vm) -> i32 {
                unsafe {
                    lua_pushinteger(vm.as_ptr(), self.0 as _);
                    1
                }
            }
        }
    };
}

impl_from_lua!(Int53, Number, lua_tointeger, lua_pushinteger);
impl_from_lua!(UInt53, Number, lua_tointeger, lua_pushinteger);
