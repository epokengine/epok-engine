# Lua core archive for the selected VM execution mode. Included by the runtime
# Makefile when sources.mk sets EPOK_LUA_VM, which only the two VM modes emit.
#
# The archive is built here rather than in the submodule for two reasons: the
# upstream Makefile writes liblua.a and liblua-noparser.a from the SAME object
# files, so the two variants cannot coexist; and nothing may be written inside
# third_party during a build. Objects land in a per-variant directory under the
# build directory, so a mode switch can never link a stale parser object.
#
# libpsyqo-lua.a is deliberately not part of this: its constructor loads a
# source bootstrap chunk and opens the standard libraries. runtime/lua_runtime.hpp
# owns the allocator and the luaI_* shims instead.

LUA_SRC_DIR := $(NUGGET_DIR)/third_party/psxlua/src

ifeq ($(EPOK_LUA_NOPARSER),1)
LUA_VARIANT := noparser
# lnoparser.c replaces the compiler with a stub that rejects text chunks.
LUA_PARSER_UNITS := lnoparser
else
LUA_VARIANT := parser
LUA_PARSER_UNITS := lcode ldump llex lparser
endif

# psxlua's CORE_O, plus lauxlib for luaL_error and the buffer helpers the core
# uses. The standard libraries (lbaselib lbitlib lcorolib ldblib lstrlib
# ltablib linit) are absent by design: a cooked build exposes no globals but
# the registered __epok_* helpers, so nothing can reference them.
LUA_CORE_UNITS := lapi lctype ldebug ldo lfunc lgc lmem lobject lopcodes \
	lstate lstring ltable ltm lundump lvm lzio llibc lauxlib
LUA_UNITS := $(LUA_CORE_UNITS) $(LUA_PARSER_UNITS)

LUA_OBJ_DIR := lua/$(LUA_VARIANT)
LUA_ARCHIVE := lua/liblua-epok-$(LUA_VARIANT).a
LUA_OBJS := $(patsubst %,$(LUA_OBJ_DIR)/%.o,$(LUA_UNITS))

# Upstream's psx target flags. -Os matches the archive the feasibility harness
# measured; ARCHFLAGS comes from Nugget so the ABI cannot drift from the rest
# of the executable.
LUA_CFLAGS = -DLUA_TARGET_PSX -DLUA_COMPAT_ALL -Os -g -Wall -Wno-attributes \
	-ffunction-sections -fdata-sections -mno-gpopt -fomit-frame-pointer \
	-fno-builtin -fno-strict-aliasing -I$(LUA_SRC_DIR) $(ARCHFLAGS)

$(LUA_OBJ_DIR)/%.o: $(LUA_SRC_DIR)/%.c
	mkdir -p $(LUA_OBJ_DIR)
	$(CC) $(LUA_CFLAGS) -c -o $@ $<

# `D` is deterministic mode: zeroed member timestamps, uid and gid. The host
# hashes every LIBRARIES entry as a build input and re-runs the inspection
# target with -B, which rebuilds this archive; without `D` the identical
# objects would produce different bytes each time and the build would be
# rejected as "inputs changed during compilation".
$(LUA_ARCHIVE): $(LUA_OBJS)
	mkdir -p $(dir $(LUA_ARCHIVE))
	$(AR) rcsD $@ $(LUA_OBJS)

LIBRARIES += $(LUA_ARCHIVE)
# common.mk expanded the link rule's prerequisites before this file was read,
# so the dependency is restated explicitly. build-inputs.mk hashes every entry
# of LIBRARIES, which requires the archive to exist when it is inspected.
$(BINDIR)$(TARGET).elf: $(LUA_ARCHIVE)
epok-build-inputs: $(LUA_ARCHIVE)

.PHONY: lua-clean
lua-clean:
	rm -rf lua
