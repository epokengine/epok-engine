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
