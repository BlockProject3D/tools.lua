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

--- @meta bp3d.util

bp3d = {}
bp3d.util = {}
bp3d.util.string = {}
bp3d.util.table = {}
bp3d.util.utf8 = {}
bp3d.util.num = {}

--- Checks if the given `src` string starts with the prefix `prefix`.
---
--- @param src string the source string.
--- @param prefix string the string to search for.
--- @return boolean whatever true if `src` starts with `prefix`, false otherwise.
bp3d.util.string.startsWith = function(src, prefix) end

--- Checks if the given `src` string ends with the suffix `suffix`.
---
--- @param src string the source string.
--- @param suffix string the string to search for.
--- @return boolean whatever true if `src` ends with `suffix`, false otherwise.
bp3d.util.string.endsWith = function(src, suffix) end

--- Checks if the given sub string `needle` can be found in `src`.
---
--- @param src string the source string.
--- @param needle string the string to search for.
--- @return boolean whatever true if `needle` was found in `src`, false otherwise.
bp3d.util.string.contains = function(src, needle) end

--- Splits the given string `src` using the separator `pattern`.
---
--- @param src string input source string.
--- @param pattern integer separator character.
--- @return string[]
bp3d.util.string.split = function(src, pattern) end

--- Capitalises the given string.
---
--- Note: This function ignores UTF-8 characters.
---
--- @param src string the string to capitalise.
--- @return string
bp3d.util.string.capitalise = function(src) end

--- De-capitalises the given string.
---
--- Note: This function ignores UTF-8 characters.
---
--- @param src string the string to decapitalise.
--- @return string
bp3d.util.string.decapitalise = function(src) end

--- Merges the keys of 1 table into another.
---
--- @param dst table the destination table.
--- @param src table the source table to merge.
bp3d.util.table.update = function(dst, src) end

--- Concatenates N source tables into a destination table.
---
--- @param dst table the destination table which should have the keys of all source tables.
--- @param ... table a list of all source table to concatenate in `dst`.
bp3d.util.table.concat = function(dst, ...) end

--- Deep-copies the table given as argument and return a new table.
---
--- @param src table input table to deep-copy.
--- @return table
bp3d.util.table.copy = function(src) end

--- @param src table input table to compute length of.
--- @return integer whatever the number of items in `src`.
--- This function is optimized to choose either the fast objlen method or a slow iterator based on if the table has
--- a hash component or not.
bp3d.util.table.count = function(src) end

--- @param src table input table.
--- @return string whatever a string listing all key-value pairs in the table for quick display purposes.
bp3d.util.table.tostring = function(src) end

--- @param src table input table.
--- @param value any value to search for.
--- @return boolean whatever true if value was found in the table, false otherwise.
bp3d.util.table.contains = function(src, value) end

--- @param src table input table.
--- @param key any key to search for.
--- @return boolean whatever true if key was found in the table, false otherwise.
bp3d.util.table.containsKey = function(src, key) end

--- Protects the given input table. A protected table is a read-only table where writing would result into a runtime
--- error.
---
--- @param src table input table.
--- @return table
bp3d.util.table.protect = function(src) end

--- Checks if the given sub string `needle` can be found in `src`.
---
--- Note: This function wil error if any of the inputs are not UTF-8 encoded.
---
--- @param src string the source string.
--- @param needle string the string to search for.
--- @return boolean whatever true if `needle` was found in `src`, false otherwise.
bp3d.util.utf8.contains = function(src, needle) end

--- Splits the given string `src` using the separator `pattern`.
---
--- Note: This function wil error if any of the inputs are not UTF-8 encoded.
---
--- @param src string input source string.
--- @param pattern string separator string.
--- @return string[]
bp3d.util.utf8.split = function(src, pattern) end

--- Replace all instances of `pattern` in the given string `src` by the replacement string `replacement`.
---
--- Note: This function wil error if any of the inputs are not UTF-8 encoded.
---
--- @param src string input source string.
--- @param pattern string search string.
--- @param replacement string replacement string
bp3d.util.utf8.replace = function(src, pattern, replacement) end

--- Count the number of unicode characters in the source string.
---
--- Note: This function wil error if any of the inputs are not UTF-8 encoded. This function does not handle unicode
--- ligatures.
---
--- @param src string input source string.
--- @return integer
bp3d.util.utf8.count = function(src) end

--- Extract a unicode character from the given source string.
---
--- Note: This function wil error if any of the inputs are not UTF-8 encoded. This function does not handle unicode
--- ligatures.
---
--- @param src string input source string.
--- @param pos integer character index.
--- @return integer
bp3d.util.utf8.charAt = function(src, pos) end

--- Checks if the given input string is valid UTF-8.
---
--- @param src string input source string.
--- @return string? whatever nil if the input string contains invalud UTF-8 codes, otherwise returns `src`.
bp3d.util.utf8.fromString = function(src) end

--- Converts the input string to a valid UTF-8 string by replacing all invalid UTF-8 codes by U+FFFD.
---
--- @param src string input source string.
--- @return string
bp3d.util.utf8.fromStringLossy = function(src) end

--- Capitalises the given string.
---
--- Note: This function wil error if any of the inputs are not UTF-8 encoded.
---
--- @param src string the string to capitalise.
--- @return string
bp3d.util.utf8.capitalise = function(src) end

--- De-capitalises the given string.
---
--- Note: This function wil error if any of the inputs are not UTF-8 encoded.
---
--- @param src string the string to decapitalise.
--- @return string
bp3d.util.utf8.decapitalise = function(src) end

--- Change case of all characters in the given input string to upper.
---
--- Note: This function wil error if any of the inputs are not UTF-8 encoded.
---
--- @param src string the input string.
--- @return string
bp3d.util.utf8.upper = function(src) end

--- Change case of all characters in the given input string to lower.
---
--- Note: This function wil error if any of the inputs are not UTF-8 encoded.
---
--- @param src string the input string.
--- @return string
bp3d.util.utf8.lower = function(src) end

--- Extracts a sub-string from the given input source string.
---
--- Note: This function wil error if `src` is not UTF-8 encoded. This function will never return a broken UTF-8 string.
--- In case `start` points inside a multi-byte UTF-8 sequence, the start position is advanced to the next valid UTF-8
--- sequence. In case `end1` points inside a multi-byte UTF-8 sequence, the end position is moved backwards to the
--- previous valid UTF-8 sequence.
---
--- @param src string the input string.
--- @param start integer start position in the input string.
--- @param end1 integer end position in the input string.
--- @return string
bp3d.util.utf8.sub = function(src, start, end1) end

--- @class bp3d.util.num
--- @field INT53_MIN number -2^52
--- @field INT53_MAX number 2^52-1
--- @field UINT53_MIN number 0
--- @field UINT53_MAX number 2^53-1
--- @field INT64_MIN number -2^63; this is actually not a number but a special LuaJIT cdata type
--- @field INT64_MAX number 2^63-1; this is actually not a number but a special LuaJIT cdata type
--- @field UINT64_MIN number 0; this is actually not a number but a special LuaJIT cdata type
--- @field UINT64_MAX number 2^64-1; this is actually not a number but a special LuaJIT cdata type
--- @field NAN number nan
--- @field EPSILON number represents the maximum possible precision of a Lua floating-point

--- Converts the input value to an integer string.
---
--- @param value any
--- @return string
bp3d.util.num.toistring = function(value) end

--- Converts the input value to an unsigned integer string.
---
--- @param value any
--- @return string
bp3d.util.num.toustring = function(value) end

--- Tests equality between two floating-point numbers.
---
--- @param a number first operand
--- @param b number second operand
--- @param epsilon number maximum difference between a and b
--- @return boolean
bp3d.util.num.eq = function(a, b, epsilon) end

--- Parse a Lua floating-point number from the input string.
---
--- If the number was successfully parsed, returns the number and nil,
--- otherwise returns nil and a string error message.
---
--- @param str string
--- @return [number?, string?]
bp3d.util.num.parsenumber = function(str) end

--- Parse a LuaJIT true 64 bits integer from the input string.
---
--- If the number was successfully parsed, returns the integer and nil,
--- otherwise returns nil and a string error message.
---
--- @param str string
--- @return [number?, string?]
bp3d.util.num.parseint64 = function(str) end

--- Parse a LuaJIT true 64 bits unsigned integer from the input string.
---
--- If the number was successfully parsed, returns the integer and nil,
--- otherwise returns nil and a string error message.
---
--- @param str string
--- @return [number?, string?]
bp3d.util.num.parseuint64 = function(str) end
