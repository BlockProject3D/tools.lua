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

use crate::decl_lib_func;
use crate::libs::Lib;
use crate::vm::function::types::RFunction;
use crate::vm::function::IntoParam;
use crate::util::Namespace;
use crate::vm::table::Table;
use crate::vm::Vm;
use bp3d_os::time::{LocalUtcOffset, MonthExt};
use bp3d_util::simple_error;
use std::time::Instant;
use time::format_description::parse;
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

enum TableOrString<'a> {
    Table(Table<'a>),
    String(String),
}

unsafe impl IntoParam for TableOrString<'_> {
    fn into_param(self, vm: &Vm) -> u16 {
        match self {
            TableOrString::Table(t) => t.into_param(vm),
            TableOrString::String(s) => s.into_param(vm),
        }
    }
}

fn get_std_offset() -> UtcOffset {
    let now = OffsetDateTime::now_utc();
    let jan = PrimitiveDateTime::new(
        Date::from_calendar_date(now.year(), Month::January, 1).unwrap(),
        Time::MIDNIGHT,
    )
    .assume_utc();
    let jul = PrimitiveDateTime::new(
        Date::from_calendar_date(now.year(), Month::July, 1).unwrap(),
        Time::MIDNIGHT,
    )
    .assume_utc();
    let offset_jan = UtcOffset::local_offset_at(jan).unwrap();
    let offset_jul = UtcOffset::local_offset_at(jul).unwrap();
    std::cmp::max(offset_jan, offset_jul)
}

const REPLACEMENTS: &[(&str, &str)] = &[
    ("[", "[["),
    ("%%", "%"),
    ("%a", "[weekday repr:short]"),
    ("%A", "[weekday repr:long]"),
    ("%b", "[month repr:short]"),
    ("%B", "[month repr:long]"),
    ("%d", "[day]"),
    ("%H", "[hour repr:24]"),
    ("%I", "[hour repr:12]"),
    ("%M", "[minute]"),
    ("%m", "[month]"),
    ("%p", "[period]"),
    ("%S", "[second]"),
    ("%w", "[weekday]"),
    ("%Y", "[year]"),
    ("%y", "[year repr:last_two]"),
];

decl_lib_func! {
    fn date<'a>(vm: &Vm, format: Option<&str>, time: Option<i64>) -> Option<TableOrString<'a>> {
        let mut format = format.unwrap_or("%c");
        let mut time = time.map(OffsetDateTime::from_unix_timestamp).unwrap_or(Ok(OffsetDateTime::now_utc())).ok()?;
        if format.starts_with('!') {
            format = &format[1..];
        } else {
            let offset = UtcOffset::local_offset_at(time)?;
            time = time.to_offset(offset);
        }
        if format == "*t" {
            let std_offset = get_std_offset();
            let mut table = Table::new(vm);
            table.set_field(c"sec", time.second()).unwrap();
            table.set_field(c"min", time.minute()).unwrap();
            table.set_field(c"hour", time.hour()).unwrap();
            table.set_field(c"day", time.day()).unwrap();
            table.set_field(c"month", time.month() as u8).unwrap();
            table.set_field(c"year", time.year()).unwrap();
            table.set_field(c"wday", time.weekday() as u8 + 1).unwrap();
            table.set_field(c"yday", time.to_julian_day()).unwrap();
            table.set_field(c"isdst", time.offset() < std_offset).unwrap();
            Some(TableOrString::Table(table))
        } else {
            let mut format = String::from(format);
            for (k, v) in REPLACEMENTS {
                format = format.replace(k, v);
            }
            let format = parse(format.as_str()).ok()?;
            time.format(&format).map(TableOrString::String).ok()
        }
    }
}

simple_error! {
    TimeFormatError {
        (impl From)Vm(crate::vm::error::Error) => "vm error: {}",
        InvalidMonthIndex(u8) => "invalid month index {}",
        (impl From)Time(time::error::ComponentRange) => "out of range error: {}"
    }
}

fn get_time_from_table(table: Table) -> Result<OffsetDateTime, TimeFormatError> {
    let year: i32 = table.get_field(c"year")?;
    let month: u8 = table.get_field(c"month")?;
    let day: u8 = table.get_field(c"day")?;
    let date = Date::from_calendar_date(
        year,
        Month::from_index(month).ok_or(TimeFormatError::InvalidMonthIndex(month))?,
        day,
    )?;
    let hour: Option<u8> = table.get_field(c"hour")?;
    let minute: Option<u8> = table.get_field(c"min")?;
    let second: Option<u8> = table.get_field(c"sec")?;
    let mut hour = hour.unwrap_or(12);
    let minute = minute.unwrap_or(0);
    let second = second.unwrap_or(0);
    let dst: Option<bool> = table.get_field(c"isdst")?;
    let dst = dst.unwrap_or(false);
    // Consider DST to be always +1H, this may not always be true but is true in most countries.
    if dst {
        hour += 1;
    }
    let time = Time::from_hms(hour, minute, second)?;
    let time = PrimitiveDateTime::new(date, time).assume_utc();
    Ok(time)
}

decl_lib_func! {
    fn time(table: Option<Table>) -> Result<i64, TimeFormatError> {
        match table {
            Some(table) => get_time_from_table(table).map(|v| v.unix_timestamp()),
            None => Ok(OffsetDateTime::now_utc().unix_timestamp())
        }
    }
}

decl_lib_func! {
    fn difftime(a: i64, b: Option<i64>) -> Option<f64> {
        let a = OffsetDateTime::from_unix_timestamp(a).ok()?;
        let b = OffsetDateTime::from_unix_timestamp(b.unwrap_or(0)).ok()?;
        Some((a - b).as_seconds_f64())
    }
}

thread_local! {
    static NOW: Instant = Instant::now();
}

decl_lib_func! {
    fn clock() -> f64 {
        NOW.with(|v| v.elapsed().as_secs_f64())
    }
}

decl_lib_func! {
    fn getenv(key: &str) -> Option<Vec<u8>> {
        std::env::var_os(key).map(|v| v.into_encoded_bytes())
    }
}

pub struct Compat;

impl Lib for Compat {
    const NAMESPACE: &'static str = "os";

    fn load(&self, namespace: &mut Namespace) -> crate::vm::Result<()> {
        namespace.add([
            ("date", RFunction::wrap(date)),
            ("time", RFunction::wrap(time)),
            ("clock", RFunction::wrap(clock)),
            ("difftime", RFunction::wrap(difftime)),
            ("getenv", RFunction::wrap(getenv)),
        ])
    }
}
