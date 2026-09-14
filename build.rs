fn main() {
    assert!(
        std::path::Path::new("third_party/nugget/psyqo/fixed-point.hh").is_file(),
        "Effect preview requires the pinned Nugget headers. Run the repository setup tool before building the editor."
    );
    let output =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo output directory"));
    std::fs::create_dir_all(output.join("EASTL")).expect("Host SDK adapter directory");
    // Only the unused text callback wrapper and display constants differ from
    // PSX. The actual timeline, effect, particle and PsyQo Q12 headers are shared.
    std::fs::write(
        output.join("EASTL/functional.h"),
        "#pragma once\n#include <functional>\nnamespace eastl {using std::function;}\n",
    )
    .unwrap();
    std::fs::write(output.join("display.hh"), "#pragma once\nnamespace epok {inline constexpr int display_width=320,display_height=240;}\n").unwrap();
    println!("cargo:rerun-if-changed=native/effect_preview.cpp");
    println!("cargo:rerun-if-changed=native/effect_preview.h");
    println!("cargo:rerun-if-changed=native/hud_preview.cpp");
    println!("cargo:rerun-if-changed=native/hud_preview.h");
    println!("cargo:rerun-if-changed=native/hud_commands.hpp");
    println!("cargo:rerun-if-changed=runtime");
    println!("cargo:rerun-if-changed=third_party/nugget/psyqo/fixed-point.hh");
    cc::Build::new()
        .cpp(true)
        .std("c++20")
        .warnings(false)
        .include(&output)
        .include("third_party/nugget")
        .include("runtime")
        .file("native/effect_preview.cpp")
        .file("native/hud_preview.cpp")
        .compile("epok_effect_preview");
    println!("cargo:rerun-if-changed=native/sequence_preview.cpp");
    println!("cargo:rerun-if-changed=native/sequence_preview.h");
    println!("cargo:rerun-if-changed=native/instrument_preview.cpp");
    println!("cargo:rerun-if-changed=native/instrument_preview.h");
    println!("cargo:rerun-if-changed=native/instrument_source_preview.cpp");
    println!("cargo:rerun-if-changed=native/instrument_source_preview.h");
    cc::Build::new()
        .cpp(true)
        .std("c++20")
        .warnings(true)
        .include("runtime")
        .file("native/sequence_preview.cpp")
        .file("native/instrument_preview.cpp")
        .file("native/instrument_source_preview.cpp")
        .compile("epok_sequence_preview");
    lua_cooker();
    println!("cargo:rerun-if-changed=resources/branding/epok.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("resources/branding/epok.ico")
            .set("ProductName", "Epok Engine")
            .set("FileDescription", "Epok PlayStation game editor")
            .compile()
            .expect("Failed to compile Epok's Windows icon resource");
    }
}

/// Builds the host bytecode cooker from the pinned `psxlua` submodule.
///
/// The parser, lexer and code generator are the submodule's own sources built
/// for this host; only the dumper is ours, because psxlua's `lua_Number` is
/// `long` and its `size_t` is 32-bit on the PlayStation but 64-bit here.
/// `cargo:rustc-cfg=epok_luac` gates `lua_bytecode::cook`: without the
/// submodule the editor still builds, and the VM bytecode mode reports the
/// missing sources as its real cause instead of silently shipping source.
fn lua_cooker() {
    let src = std::path::Path::new("third_party/nugget/third_party/psxlua/src");
    println!("cargo:rustc-check-cfg=cfg(epok_luac)");
    println!("cargo:rerun-if-changed=native/lua/epok_luac.c");
    println!("cargo:rerun-if-changed=native/lua/epok_ldump32.c");
    println!("cargo:rerun-if-changed=native/lua/epok_luac.h");
    println!("cargo:rerun-if-changed=native/lua/epok_luac_host.h");
    println!("cargo:rerun-if-changed={}/lparser.c", src.display());
    if !src.join("lparser.c").is_file() {
        return;
    }
    // The core plus the parser half of psxlua's `psx` object list. The standard
    // libraries are deliberately absent: nothing here executes a chunk.
    // `ldump.c` is included only because `lapi.c` references `luaU_dump`.
    const UNITS: [&str; 22] = [
        "lapi", "lauxlib", "lcode", "lctype", "ldebug", "ldo", "ldump", "lfunc", "lgc", "llex",
        "llibc", "lmem", "lobject", "lopcodes", "lparser", "lstate", "lstring", "ltable", "ltm",
        "lundump", "lvm", "lzio",
    ];
    let mut build = cc::Build::new();
    build
        .warnings(false)
        .define("LUA_COMPAT_ALL", None)
        .define("LUA_USER_H", Some("\"epok_luac_host.h\""))
        .include(src)
        .include("native/lua")
        .file("native/lua/epok_luac.c")
        .file("native/lua/epok_ldump32.c");
    // No LUA_TARGET_PSX: the host build uses its own libc, which is what makes
    // the parser usable here at all.
    for unit in UNITS {
        build.file(src.join(format!("{unit}.c")));
    }
    build.compile("epok_lua_cooker");
    println!("cargo:rustc-cfg=epok_luac");
}
