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

--- @meta bp3d.lua.shell

bp3d = {}
bp3d.lua = {}
bp3d.lua.shell = {}

--- Looks for a value named `name` in _G and build autocompletions to be sent to the frontend for rendering purposes.
---
--- @raises runtime error if the value with name `name` cannot be found.
--- @param name string the name of the key in _G to build autocompletions for.
--- @param metatable boolean true to enable listing recursive metatables, false otherwise.
bp3d.lua.shell.buildCompletions = function(name, metatable) end

--- Removes all autocompletions which were added for `name`.
---
--- @raises runtime error if the value with name `name` cannot be found.
--- @param name string the name of the key in _G to remove autocompletions for.
--- @param metatable boolean true to enable listing recursive metatables, false otherwise.
bp3d.lua.shell.deleteCompletions = function(name, metatable) end

--- Schedules a lua thread to be resumed/started after a specified time as a whole number in milliseconds.
---
---@param thread thread the thread to resume/start.
---@param after_ms integer 32 bits unsigned integer number of milliseconds after now at which to run the thread.
bp3d.lua.shell.scheduleIn = function(thread, after_ms) end

--- Schedules a lua thread to be resumed/started at a regular time interval specified as a whole number in milliseconds.
---
---@param thread thread the thread to resume/start.
---@param period_ms integer 32 bits unsigned integer interval in milliseconds at which to run the thread.
bp3d.lua.shell.schedulePeriodically = function(thread, period_ms) end

--- Binds a Lua function to the given event name.
---
---@param name string the name of the event to bind to.
---@param func function the lua function to run every time this event is raised.
bp3d.lua.shell.bindEvent = function(name, func) end

--- Unbinds the Lua function from the given event name.
---
---@param name string the name of the event to unbind the lua function from.
bp3d.lua.shell.unbindEvent = function(name) end

--- Requests exit of the shell application from lua code.
bp3d.lua.shell.requestExit = function() end
