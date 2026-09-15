//! Host bytecode cooker for the pinned psxlua ABI (M4).
//!
//! `build.rs` compiles the submodule's own parser for this host together with
//! `native/lua/epok_ldump32.c`, a dumper that writes the two host-width
//! quantities (`size_t` and, on LP64 hosts, psxlua's `long` `lua_Number`)
//! at the target's 32-bit width. The output therefore has to be byte-identical
//! to an on-target `luaU_dump`, which
//! `tests/integration/verify_lua_vm_abi.py` checks.
#![allow(dead_code)] // Consumed by lua_vm once M4 lands.

/// `"\x1bLua"`, version 5.2, format 0, little endian, `sizeof(int)`,
/// `sizeof(size_t)`, `sizeof(Instruction)` and `sizeof(lua_Number)` all 4,
/// integral numbers, then `LUAC_TAIL`. Mirrored by `EPOK_LUAC_HEADER`.
pub const HEADER: [u8; 18] = [
    0x1B, 0x4C, 0x75, 0x61, 0x52, 0x00, 0x01, 0x04, 0x04, 0x04, 0x04, 0x01, 0x19, 0x93, 0x0D, 0x0A,
    0x1A, 0x0A,
];

/// Debug information is kept: Lua runtime errors then name the chunk and the
/// line, which is the only way a shipped VM build can point at the authored
/// `.lua`. `docs/lua-vm-runtime.md` records the size this costs.
pub const KEEP_DEBUG_INFO: bool = true;

pub const MISSING_SOURCES: &str = "psxlua sources are missing; run the repository setup to initialize third_party/nugget/third_party/psxlua";

/// Compiles one normalized chunk to target bytecode. `chunk_name` is the Lua
/// chunk name and appears in runtime error messages.
/// The predicate the no-parser runtime applies before `luaU_undump` accepts a
/// payload: every byte of the pinned header has to match.
pub fn is_pinned_bytecode(bytes: &[u8]) -> bool {
    bytes.starts_with(&HEADER)
}

pub fn cook(chunk_name: &str, source: &str) -> Result<Vec<u8>, String> {
    let bytes = cook_with(chunk_name, source, !KEEP_DEBUG_INFO)?;
    if !is_pinned_bytecode(&bytes) {
        return Err(format!(
            "cooked bytecode for {chunk_name} does not carry the pinned psxlua header; the host cooker and the target ABI disagree"
        ));
    }
    Ok(bytes)
}

#[cfg(epok_luac)]
pub fn cook_with(chunk_name: &str, source: &str, strip: bool) -> Result<Vec<u8>, String> {
    use std::os::raw::{c_char, c_int, c_void};
    unsafe extern "C" {
        fn epok_luac_cook(
            name: *const c_char,
            src: *const c_char,
            len: usize,
            strip: c_int,
            ctx: *mut c_void,
            writer: extern "C" fn(*mut c_void, *const c_void, usize) -> c_int,
            err: *mut c_char,
            errcap: usize,
        ) -> c_int;
    }
    extern "C" fn write(ctx: *mut c_void, bytes: *const c_void, size: usize) -> c_int {
        // SAFETY: `ctx` is the `Vec` below, alive for the whole call, and the C
        // side never keeps the pointer past the callback.
        let out = unsafe { &mut *ctx.cast::<Vec<u8>>() };
        out.extend_from_slice(unsafe { std::slice::from_raw_parts(bytes.cast::<u8>(), size) });
        0
    }
    // Lua chunk names are C strings; an embedded NUL would truncate the name
    // the runtime reports, so reject it rather than cook a mislabelled chunk.
    let name = std::ffi::CString::new(chunk_name)
        .map_err(|_| format!("chunk name {chunk_name:?} contains a NUL byte"))?;
    let mut out: Vec<u8> = Vec::new();
    let mut error = vec![0 as c_char; 512];
    let status = unsafe {
        epok_luac_cook(
            name.as_ptr(),
            source.as_ptr().cast::<c_char>(),
            source.len(),
            c_int::from(strip),
            std::ptr::addr_of_mut!(out).cast::<c_void>(),
            write,
            error.as_mut_ptr(),
            error.len(),
        )
    };
    if status != 0 {
        let message = unsafe { std::ffi::CStr::from_ptr(error.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        return Err(if message.is_empty() {
            format!("the pinned psxlua compiler rejected {chunk_name}")
        } else {
            message
        });
    }
    Ok(out)
}

#[cfg(not(epok_luac))]
pub fn cook_with(_chunk_name: &str, _source: &str, _strip: bool) -> Result<Vec<u8>, String> {
    Err(MISSING_SOURCES.into())
}

/// True when this build can cook bytecode at all. Callers use it to report the
/// missing submodule rather than a script problem.
pub const fn available() -> bool {
    cfg!(epok_luac)
}

#[cfg(test)]
mod tests {
    const CHUNK: &str = "local C = {}\nfunction C.tick(self, epok_p0)\n  local epok_l0 = __epok_iadd(epok_p0, 1)\n  __epok_setf(self, 0, epok_l0)\n  return epok_l0\nend\nreturn C\n";

    #[test]
    fn lua_bytecode_cooks_the_pinned_header_and_keeps_debug_information() {
        if !super::available() {
            eprintln!("psxlua submodule absent; cooker test skipped");
            assert!(super::cook("@t.lua", CHUNK).is_err());
            return;
        }
        let cooked = super::cook("@t.lua", CHUNK).unwrap();
        assert_eq!(&cooked[..18], &super::HEADER);
        // Debug information is kept by default, so the chunk name survives into
        // the payload and a runtime error can quote the authored file.
        assert!(
            cooked.windows(7).any(|w| w == b"@t.lua\0"),
            "the default cook must retain the chunk name"
        );
        let stripped = super::cook_with("@t.lua", CHUNK, true).unwrap();
        assert_eq!(&stripped[..18], &super::HEADER);
        assert!(
            stripped.len() < cooked.len(),
            "stripping must remove bytes: {} vs {}",
            stripped.len(),
            cooked.len()
        );
    }

    #[test]
    fn lua_bytecode_rejects_unparsable_and_out_of_range_chunks() {
        if !super::available() {
            return;
        }
        let error = super::cook("@t.lua", "local C = {}\nfunction C.\nreturn C\n").unwrap_err();
        assert!(error.contains("t.lua"), "{error}");
        // LP64 hosts parse through a wider `long`, so the target dumper must
        // reject a value it cannot encode. Windows already uses the target's
        // 32-bit `long`; authored values are range-checked by `lua_frontend`
        // before this internal cooker receives normalized source.
        if std::mem::size_of::<std::os::raw::c_long>() > 4 {
            let error = super::cook("@t.lua", "return 4294967296\n").unwrap_err();
            assert!(error.contains("32-bit lua_Number"), "{error}");
        } else {
            let cooked = super::cook("@t.lua", "return 2147483647\n").unwrap();
            assert!(super::is_pinned_bytecode(&cooked));
        }
    }

    /// The no-parser runtime hands a payload straight to `luaU_undump`, which
    /// accepts it only when all 18 header bytes match the pinned ABI. Cooking
    /// applies the same predicate before shipping, so a payload whose header
    /// drifted by a single byte never reaches the console.
    #[test]
    fn lua_bytecode_rejects_a_payload_whose_header_byte_drifted() {
        if !super::available() {
            return;
        }
        let cooked = super::cook("@t.lua", CHUNK).unwrap();
        assert!(super::is_pinned_bytecode(&cooked));
        for index in 0..super::HEADER.len() {
            let mut corrupt = cooked.clone();
            corrupt[index] ^= 0x01;
            assert!(
                !super::is_pinned_bytecode(&corrupt),
                "header byte {index} may not be ignored"
            );
        }
        assert!(!super::is_pinned_bytecode(
            &cooked[..super::HEADER.len() - 1]
        ));
    }

    #[test]
    fn lua_bytecode_text_chunks_are_distinguishable_from_cooked_ones() {
        // A no-parser build accepts a chunk only when it opens with this exact
        // header, so the emitter can tell before shipping whether a payload
        // would be rejected as text.
        assert!(!CHUNK.as_bytes().starts_with(&super::HEADER));
        assert_ne!(super::HEADER[0], b'l');
    }
}
