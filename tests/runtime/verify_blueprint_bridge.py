"""Run bridge protocol tests in the pinned Redux LuaJIT DLL, without a desktop/emulator."""
import ctypes
import os
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]

def main():
    directory=ROOT/".tools/redux"
    if not (directory/"lua51.dll").is_file():
        raise SystemExit("Run SDK setup first: the pinned Redux LuaJIT DLL is required.")
    with os.add_dll_directory(str(directory)):
        lua=ctypes.CDLL(str(directory/"lua51.dll"))
        lua.luaL_newstate.restype=ctypes.c_void_p
        lua.luaL_openlibs.argtypes=[ctypes.c_void_p]
        lua.lua_pushstring.argtypes=[ctypes.c_void_p,ctypes.c_char_p]
        lua.lua_setfield.argtypes=[ctypes.c_void_p,ctypes.c_int,ctypes.c_char_p]
        lua.luaL_loadfile.argtypes=[ctypes.c_void_p,ctypes.c_char_p]
        lua.lua_pcall.argtypes=[ctypes.c_void_p,ctypes.c_int,ctypes.c_int,ctypes.c_int]
        lua.lua_tolstring.argtypes=[ctypes.c_void_p,ctypes.c_int,ctypes.c_void_p]
        lua.lua_tolstring.restype=ctypes.c_char_p
        lua.lua_close.argtypes=[ctypes.c_void_p]
        state=lua.luaL_newstate()
        try:
            lua.luaL_openlibs(state)
            lua.lua_pushstring(state,str(ROOT/"integrations/pcsx-redux/bridge.lua").encode())
            lua.lua_setfield(state,-10002,b"EPOK_TEST_BRIDGE")
            result=lua.luaL_loadfile(state,str(ROOT/"tests/runtime/test_blueprint_bridge.lua").encode())
            if result==0:
                result=lua.lua_pcall(state,0,0,0)
            if result:
                raise AssertionError(lua.lua_tolstring(state,-1,None).decode())
        finally:
            lua.lua_close(state)

if __name__=="__main__":
    main()
