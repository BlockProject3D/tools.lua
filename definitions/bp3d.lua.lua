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

--- @meta bp3d.lua

bp3d = {}

--- @class bp3d.lua
--- @field name string The name of the Lua engine used.
--- @field version string The version string of the Lua engine following SemVer notation.
--- @field patches string[] The list of patches applied to the underlying LuaJIT interpreter.
bp3d.lua = {}

--- Protect call to a Lua function and extract traceback information in case of error.
---
--- @param func function the function to call in a protected environment.
--- @return [boolean, (any[] | string)] whatever true if the call succeeded, false otherwise followed by the error backtrace.
bp3d.lua.pcall = function(func) end

--- Runs a Lua string and raises an error if the execution failed.
---
--- @raises syntax or runtime error.
--- @param s string to run.
--- @param chunkname string? optional chunkname.
--- @return any[] whatever outputs returned by the function which was executed.
bp3d.lua.runString = function(s, chunkname) end

--- Loads a Lua string and raises an error on broken syntax.
--- This function does not run the string.
---
--- @raises syntax error.
--- @param s string to load/compile.
--- @param chunkname string? optional chunkname.
--- @return function whatever the compiled function.
bp3d.lua.loadString = function(s, chunkname) end

--- Loads a Lua file and raises an error on broken syntax.
--- This function does not run the string.
---
--- # Sandboxing
---
--- This function uses a chroot to limit the location of the file which can loaded.
--- This avoids arbitrary code execution on files which the user does not whish to make visible to the Lua engine.
---
--- @raises syntax error.
--- @param path string | Path path to the file to load/compile.
--- @return function whatever the compiled function.
bp3d.lua.loadFile = function(path) end

--- Runs a Lua file and raises an error if the execution failed.
---
--- # Sandboxing
---
--- This function uses a chroot to limit the location of the file which can loaded.
--- This avoids arbitrary code execution on files which the user does not whish to make visible to the Lua engine.
---
--- @raises syntax or runtime error.
--- @param path string | Path path to the file to load/compile and run.
--- @return any[] whatever outputs returned by the function which was executed.
bp3d.lua.runFile = function(path) end

--- Runs a Lua file and raises an error if the execution failed.
--- Unlike runFile this function allows to omit script file extension and uses a search algorithm to be defined by the
--- host application instead of the bp3d-lua engine.
--- The search algorithm is optional, if no search algorithm is provided by the host application, user of the bp3d-lua
--- engine, then this function will be set to nil.
---
--- # Sandboxing
---
--- To control which directories are allowed to contain runnable lua files, the search algorithm is left to the host
--- application.
---
--- @raises syntax or runtime error.
--- @param path string path to the script library file to load/compile and run. Path separator is '.' like standard
---                    require function.
--- @return any[] whatever outputs returned by the function which was executed.
bp3d.lua.require = function(path) end

bp3d.lua.module = {}

--- Loads a native module into the current underlying LuaJIT interpreter.
--- File extensions and other platform specific prefixes are pre-applied by the underlying search algorithm and must
--- not be specified in the lib or plugin arguments to this function.
---
--- # Sandboxing
---
--- This function limits the modules search paths to a set of folders specified at the application level.
--- This limits arbitrary native code execution and essentially forbids arbitrary native code injection by Lua code.
---
--- @param lib string the name of the library to load.
--- @param plugin string the name of the plugin in the native library to load.
bp3d.lua.module.load = function(lib, plugin) end
