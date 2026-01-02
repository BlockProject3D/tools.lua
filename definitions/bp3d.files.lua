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

--- @meta bp3d.files

bp3d = {}
bp3d.files = {}

--- @class Path
Path = {}

--- Join this path with a new component.
---
--- @param other string | Path path component to join with.
--- @return Path
function Path:join(other) end

--- Changes the extension of this path.
---
--- @param ext string the new extension to set.
--- @return Path
function Path:withExtension(ext) end

--- Changes the file name of this path.
---
--- @param name string the new file name to set.
--- @return Path
function Path:withName(name) end

--- @return string whatever the file name.
function Path:name() end

--- @return string whatever the file extension.
function Path:extension() end

--- Creates a new Path object.
---
--- @param path string the path to wrap.
--- @return Path
Path.new = function(path) end

--- @class File
File = {}

--- Read a block of data from the file. Returns a byte string of the content.
---
--- @param size number number of bytes to read.
--- @return string
function File:read(size) end

--- Writes the given block of data to the file. Returns the number of bytes written.
---
--- @param data string byte string representing the block of data.
--- @return number
function File:write(data) end

--- Seeks from the start of the file.
---
--- @param pos number (uint64_t LuaJIT cdata type)
--- @return number uint64_t LuaJIT cdata type
function File:seekFromStart(pos) end

--- Seeks from the end of the file.
---
--- @param pos number (int64_t LuaJIT cdata type)
--- @return number uint64_t LuaJIT cdata type
function File:seekFromEnd(pos) end

--- Seeks from the current cursor position.
---
--- @param pos number (int64_t LuaJIT cdata type)
--- @return number uint64_t LuaJIT cdata type
function File:seekFromCursor(pos) end

--- Returns the size of the file.
---
--- @return number uint64_t LuaJIT cdata type
function File:size() end

--- Opens a new file for read/write or append.
---
--- @param path string | Path the path of the file.
--- @param mode string the file mode, r for read, w for write and a for append.
--- @return File
File.open = function(path, mode) end

--- Creates a new file for writing.
---
--- @param path string | Path the path of the file.
--- @return File
File.create = function(path) end

--- Read a text file.
---
--- @param path string | Path the path to read from.
--- @return string
bp3d.files.readText = function(path) end

--- Write a text file.
---
--- @param path string | Path the path to write to.
--- @param data string the file content.
bp3d.files.writeText = function(path, data) end

--- Copy a file.
---
--- @param src string | Path the source path.
--- @param dst string | Path the destination path.
bp3d.files.copyFile = function(src, dst) end

--- Creeates a symlink.
---
--- @param src string | Path the source path.
--- @param dst string | Path the destination path.
bp3d.files.symlink = function(src, dst) end

--- Creeates the directory at the specified path.
---
--- @param path string | Path
bp3d.files.deleteDir = function(path) end

--- Creeates a directory at the specified path.
---
--- @param path string | Path
bp3d.files.createDir = function(path) end

--- Returns true if the path exists.
---
--- @param path string | Path
--- @return boolean
bp3d.files.exists = function(path) end

--- List files and folders under the specified directory.
---
--- @param path string | Path
--- @return [{ path: Path, name: string, type: "dir" | "file" | "symlink" | "other" }]
bp3d.files.list = function(path) end

--- Returns the permissions of the specified file path.
---
--- @param path string | Path
--- @return { r: boolean, w: boolean, x: boolean }
bp3d.files.access = function(path) end

--- Deletes a single file.
---
--- @param path string | Path file path.
bp3d.files.delete = function(path) end

--- Renames a file. This function fails if src does not exist or if dst already exists.
---
--- @param src string | Path the source path.
--- @param dst string | Path the destination path.
bp3d.files.rename = function(src, dst) end
