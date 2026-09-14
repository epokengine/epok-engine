-- Normalized Epok chunk, in exactly the shape src/lua_vm.rs emits (contract
-- §10.2): one table of functions, every arithmetic operation a call into a
-- registered helper, Bool as a Lua boolean inside the chunk and 0/1 across the
-- boundary. It is handwritten so the ABI check has a fixed input that does not
-- move when the emitter's formatting changes; `verify_lua_vm_abi.py` also cooks
-- this exact text on the host and dumps it on target, and the two must match
-- byte for byte.
local C = {}
function C.probe_iadd(self, epok_p0)
  return __epok_iadd(epok_p0, 1)
end
function C.probe_idiv(self, epok_p0)
  return __epok_idiv(epok_p0, 0)
end
function C.probe_fmul(self, epok_p0)
  return __epok_fmul(epok_p0, -1229)
end
function C.probe_ineg(self, epok_p0)
  return __epok_ineg(epok_p0)
end
function C.probe_ult(self, epok_p0)
  return __epok_ult(1, (-2147483647 - 1))
end
function C.probe_bool(self, epok_p0)
  local epok_l0
  epok_l0 = (__epok_getf(self, 1) ~= 0)
  __epok_setf(self, 1, (not epok_l0) and 1 or 0)
  return (__epok_getf(self, 1) ~= 0)
end
function C.probe_call(self, epok_p0)
  return __epok_call(self, 0, epok_p0)
end
return C
