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

use bp3d_os::time::{LocalOffsetDateTime, MonthExt};
use bp3d_util::simple_error;
use time::error::{ComponentRange, Format, InvalidFormatDescription};
use time::format_description::parse;
use time::{Date, Duration, Month, OffsetDateTime, PrimitiveDateTime, UtcOffset};
use crate::{decl_lib_func, decl_userdata};
use crate::libs::Lib;
use crate::vm::function::IntoParam;
use crate::vm::function::types::RFunction;
use crate::vm::namespace::Namespace;
use crate::vm::table::Table;
use crate::vm::Vm;

simple_error! {
    FormatError {
        (impl From)InvalidDescription(InvalidFormatDescription) => "invalid format description: {}",
        (impl From)Format(Format) => "format error: {}"
    }
}

struct Wrapper(OffsetDateTime);

decl_userdata! {
    impl Wrapper {
        fn format(this: &Wrapper, format: &str) -> Result<String, FormatError> {
            let desc = parse(format)?;
            let str = this.0.format(&desc)?;
            Ok(str)
        }

        fn __add(this: &Wrapper, duration: f64) -> Option<Wrapper> {
            this.0.checked_add(Duration::seconds_f64(duration)).map(Wrapper)
        }

        fn __sub(this: &Wrapper, other: &Wrapper) -> f64 {
            (this.0 - other.0).as_seconds_f64()
        }

        fn __gt(this: &Wrapper, other: &Wrapper) -> bool {
            this.0 > other.0
        }

        fn __ge(this: &Wrapper, other: &Wrapper) -> bool {
            this.0 >= other.0
        }

        fn __lt(this: &Wrapper, other: &Wrapper) -> bool {
            this.0 < other.0
        }

        fn __le(this: &Wrapper, other: &Wrapper) -> bool {
            this.0 <= other.0
        }

        fn get_date<'a>(this: &Wrapper, vm: &Vm) -> crate::vm::Result<Table<'a>> {
            let mut table = Table::with_capacity(vm, 0, 3);
            table.set_field(c"year", this.0.year())?;
            table.set_field(c"month", this.0.month() as u8)?;
            table.set_field(c"day", this.0.day())?;
            Ok(table)
        }

        fn get_time<'a>(this: &Wrapper, vm: &Vm) -> crate::vm::Result<Table<'a>> {
            let mut table = Table::with_capacity(vm, 0, 3);
            table.set_field(c"hour", this.0.hour())?;
            table.set_field(c"minute", this.0.minute())?;
            table.set_field(c"second", this.0.second())?;
            Ok(table)
        }

        fn get_offset<'a>(this: &Wrapper, vm: &Vm) -> crate::vm::Result<Table<'a>> {
            let mut table = Table::with_capacity(vm, 0, 3);
            table.set_field(c"hours", this.0.offset().whole_hours())?;
            table.set_field(c"minutes", this.0.offset().whole_minutes())?;
            table.set_field(c"seconds", this.0.offset().whole_seconds())?;
            Ok(table)
        }
    }
}

unsafe impl IntoParam for OffsetDateTime {
    fn into_param(self, vm: &Vm) -> u16 {
        Wrapper(self).into_param(vm)
    }
}

decl_lib_func! {
    fn now_utc() -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

decl_lib_func! {
    fn now_local() -> Option<OffsetDateTime> {
        OffsetDateTime::now_local()
    }
}

decl_lib_func! {
    fn from_unix_timestamp(timestamp: i64) -> Result<OffsetDateTime, ComponentRange> {
        OffsetDateTime::from_unix_timestamp(timestamp)
    }
}

simple_error! {
    DateTimeError {
        (impl From)Vm(crate::vm::error::Error) => "vm error: {}",
        InvalidMonthIndex(u8) => "invalid month index {}",
        (impl From)Time(ComponentRange) => "out of range error: {}"
    }
}

decl_lib_func! {
    fn new(table: Table) -> Result<OffsetDateTime, DateTimeError> {
        let year: i32 = table.get_field(c"year")?;
        let month: u8 = table.get_field(c"month")?;
        let day: u8 = table.get_field(c"day")?;
        let date = Date::from_calendar_date(year, Month::from_index(month).ok_or(DateTimeError::InvalidMonthIndex(month))?, day)?;
        let hour: Option<u8> = table.get_field(c"hour")?;
        let minute: Option<u8> = table.get_field(c"min")?;
        let second: Option<u8> = table.get_field(c"sec")?;
        let hour = hour.unwrap_or(12);
        let minute = minute.unwrap_or(0);
        let second = second.unwrap_or(0);
        let time = time::Time::from_hms(hour, minute, second)?;
        let offset: Option<Table> = table.get_field(c"offset")?;
        if let Some(offset) = offset {
            let offset_hours: i8 = offset.get_field(c"hours")?;
            let offset_minutes: i8 = offset.get_field(c"minutes")?;
            let offset_seconds: i8 = offset.get_field(c"seconds")?;
            let offset = UtcOffset::from_hms(offset_hours, offset_minutes, offset_seconds)?;
            Ok(OffsetDateTime::new_in_offset(date, time, offset))
        } else {
            Ok(PrimitiveDateTime::new(date, time).assume_utc())
        }
    }
}

pub struct Time;

impl Lib for Time {
    const NAMESPACE: &'static str = "bp3d.os.time";

    fn load(&self, namespace: &mut Namespace) -> crate::vm::Result<()> {
        namespace.vm().register_userdata::<Wrapper>(crate::vm::userdata::case::Camel)?;
        namespace.add([
            ("nowUtc", RFunction::wrap(now_utc)),
            ("nowLocal", RFunction::wrap(now_local)),
            ("fromUnixTimestamp", RFunction::wrap(from_unix_timestamp)),
            ("new", RFunction::wrap(new))
        ])
    }
}
