-- Copyright (c) 2025, BlockProject 3D
--
-- All rights reserved.
--
-- Redistribution and use in source and binary forms, with or without modification,
-- are permitted provided that the following conditions are met:
--
--     * Redistributions of source code must retain the above copyright notice,
--       this list of conditions and the following disclaimer.
--     * Redistributions in binary form must reproduce the above copyright notice,
--       this list of conditions and the following disclaimer in the documentation
--       and/or other materials provided with the distribution.
--     * Neither the name of BlockProject 3D nor the names of its contributors
--       may be used to endorse or promote products derived from this software
--       without specific prior written permission.
--
-- THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
-- "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
-- LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
-- A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
-- CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
-- EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
-- PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
-- PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
-- LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
-- NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
-- SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

--- @meta bp3d.os

bp3d = {}
bp3d.os = {}

bp3d.os.instant = {}

--- Creates a new instance of a Instant object.
--- @return Instant whatever a new Instant object.
bp3d.os.instant.now = function() end

--- @class Instant
Instant = {}

--- @return number whatever number of seconds since the creation of this object.
function Instant:elapsed() end

bp3d.os.time = {}

--- @return OffsetDateTime
bp3d.os.time.nowUtc = function() end

--- @return OffsetDateTime
bp3d.os.time.nowLocal = function() end

--- @class Offset
--- @field hours    integer
--- @field minutes  integer
--- @field seconds  integer
Offset = {}

--- @class Time
--- @field hour     integer
--- @field minute   integer
--- @field second   integer
Time = {}

--- @class Date
--- @field year     integer
--- @field month    integer
--- @field day      integer
Date = {}

--- @class RawComponents
--- @field year     integer
--- @field month    integer
--- @field day      integer
--- @field hour     integer?
--- @field min      integer?
--- @field sec      integer?
--- @field offset   Offset?
RawComponents = {}

bp3d.os.time.OffsetDateTime = {}

--- Creates a new OffsetDateTime from its raw components.
---
--- @param table RawComponents manual definition of a date time with an optional offset.
--- @return OffsetDateTime
bp3d.os.time.OffsetDateTime.new = function(table) end

--- Constructs a new OffsetDateTime from a unix timestamp in seconds.
---
--- @param timestamp integer the unix timestamp in seconds.
--- @return OffsetDateTime
bp3d.os.time.OffsetDateTime.fromUnixTimestamp = function(timestamp) end

--- @class OffsetDateTime
OffsetDateTime = {}

--- Formats this OffsetDateTime following a given format string.
--- The format string should match with the semantics of the Rust time crate.
---
--- @param format string the format string.
--- @return string whatever formatted string.
function OffsetDateTime:format(format) end

--- @param duration number duration in seconds.
--- @return OffsetDateTime
function OffsetDateTime:__add(duration) end

--- @param other OffsetDateTime other operand to subtract with.
--- @return number whatever the dureation in seconds of (self - other).
function OffsetDateTime:__sub(other) end

--- @param other OffsetDateTime other operand to compare with.
--- @return boolean whatever true if self > other, false otherwise.
function OffsetDateTime:__gt(other) end

--- @param other OffsetDateTime other operand to compare with.
--- @return boolean whatever true if self >= other, false otherwise.
function OffsetDateTime:__ge(other) end

--- @param other OffsetDateTime other operand to compare with.
--- @return boolean whatever true if self < other, false otherwise.
function OffsetDateTime:__lt(other) end

--- @param other OffsetDateTime other operand to compare with.
--- @return boolean whatever true if self <= other, false otherwise.
function OffsetDateTime:__le(other) end

--- @return Date whatever the date component of this OffsetDateTime.
function OffsetDateTime:getDate() end

--- @return Time whatever the time component of this OffsetDateTime.
function OffsetDateTime:getTime() end

--- @return Offset whatever the UTC offset component of this OffsetDateTime.
function OffsetDateTime:getOffset() end
