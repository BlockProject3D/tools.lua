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
