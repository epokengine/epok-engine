//! Invoke the pinned host extractor and invalidate its cache by transitive content.
use crate::{
    project,
    reflection_schema::{self as schema, Manifest, Request},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub fn script_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    fn visit(path: &Path, canonical: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
        if !path.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_type().map_err(|e| e.to_string())?.is_symlink() {
                return Err(format!(
                    "Script links are unsupported: {}",
                    entry.path().display()
                ));
            }
            let path = entry.path();
            let resolved = fs::canonicalize(&path).map_err(|e| e.to_string())?;
            if !resolved.starts_with(canonical) {
                return Err("Scripts must remain inside the project".into());
            }
            if path.is_dir() {
                visit(&path, canonical, output)?;
            } else {
                output.push(path);
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    visit(
        &root.join("assets/scripts"),
        &fs::canonicalize(root).map_err(|e| e.to_string())?,
        &mut files,
    )?;
    files.sort();
    Ok(files)
}

fn hash_file(path: &Path) -> Result<String, String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?)
    ))
}
fn clang_path(path: &Path) -> String {
    path.to_string_lossy()
        .strip_prefix("\\\\?\\")
        .unwrap_or(&path.to_string_lossy())
        .replace('\\', "/")
}

#[derive(Serialize, Deserialize)]
struct Cache {
    key: String,
    manifest: Manifest,
}

fn tool_path() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut directory = exe.parent().ok_or("Editor directory unavailable")?;
    if directory.file_name().is_some_and(|v| v == "deps") {
        directory = directory.parent().unwrap();
    }
    Ok(directory.join(if cfg!(windows) {
        "epok-header-tool.exe"
    } else {
        "epok-header-tool"
    }))
}

pub fn discover(root: &Path) -> Result<Manifest, String> {
    let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    let files = script_files(&root)?;
    let headers = files
        .iter()
        .filter(|p| {
            p.extension()
                .is_some_and(|v| v == "hpp" || v == "hh" || v == "h")
        })
        .collect::<Vec<_>>();
    let folder = root.join(".epok/reflection");
    let runtime = folder.join("runtime");
    project::stage_runtime(&runtime)?;
    crate::hud::stage(&runtime, &Default::default())?;
    project::write_changed(
        &runtime.join("display.hh"),
        crate::settings::rendering(&root)?.header()?.as_bytes(),
    )?;
    let config = project::Config::load(&root)?;
    let sdk = PathBuf::from(&config.nugget);
    let compiler = PathBuf::from(&config.toolchain_bin).join(if cfg!(windows) {
        "mipsel-none-elf-g++.exe"
    } else {
        "mipsel-none-elf-g++"
    });
    let compiler =
        crate::dependencies::find_executable(&compiler.to_string_lossy()).ok_or_else(|| {
            format!(
                "MIPS compiler missing: {}. Open Editor Preferences > Dependencies.",
                compiler.display()
            )
        })?;
    let mut query = Command::new(&compiler);
    crate::pipeline::quiet(&mut query);
    let includes = query
        .args(["-E", "-x", "c++", "-v", "-"])
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("Query MIPS compiler {}: {e}", compiler.display()))?;
    if !includes.status.success() {
        return Err(String::from_utf8_lossy(&includes.stderr).into());
    }
    let include_text = String::from_utf8_lossy(&includes.stderr);
    let search = include_text
        .split("#include <...> search starts here:")
        .nth(1)
        .and_then(|v| v.split("End of search list.").next())
        .ok_or("MIPS compiler did not report its C++ include paths")?;
    let mut arguments = [
        "-x",
        "c++",
        "--target=mipsel-none-elf",
        "-march=mips1",
        "-mabi=32",
        "-EL",
        "-mfp32",
        "-fno-pic",
        "-mno-abicalls",
        "-ffreestanding",
        "-fno-builtin",
        "-fno-strict-aliasing",
        "-fno-exceptions",
        "-fno-rtti",
        "-std=c++20",
        "-DEPOK_REFLECTION",
        "-Werror=ignored-attributes",
        "-D__UINT64_C(c)=c##ULL",
        "-D__INT64_C(c)=c##LL",
    ]
    .map(String::from)
    .to_vec();
    for path in [
        &runtime,
        &root.join("assets/scripts"),
        &sdk,
        &sdk.join("third_party/EASTL/include"),
        &sdk.join("third_party/EABase/include/Common"),
    ] {
        arguments.push(format!("-I{}", clang_path(path)));
    }
    for path in search.lines().map(str::trim).filter(|v| !v.is_empty()) {
        arguments.extend(["-isystem".into(), path.into()]);
    }
    let mut source = "#include \"epok.hpp\"\n".to_string();
    for header in headers {
        let path = clang_path(header);
        if path.contains(['"', '\n', '\r']) {
            return Err("Header paths may not contain quotes or line breaks".into());
        }
        source += &format!("#include \"{path}\"\n");
    }
    let source_path = folder.join("classes.cpp");
    project::write_changed(&source_path, source.as_bytes())?;
    let request = Request {
        source: PathBuf::from(clang_path(&source_path)),
        arguments,
    };
    let request_bytes = crate::document::to_vec(&request).map_err(|e| e.to_string())?;
    let request_path = folder.join("Reflection.epokrequest");
    project::write_changed(&request_path, &request_bytes)?;
    project::write_changed(&folder.join("compile_commands.json"),serde_json::to_string_pretty(&serde_json::json!([{"directory":root,"file":source_path,"arguments":std::iter::once("clang++".to_owned()).chain(request.arguments.clone()).chain([source_path.to_string_lossy().into_owned()]).collect::<Vec<_>>()}])).map_err(|e|e.to_string())?.as_bytes())?;
    let tool = tool_path()?;
    if !tool.is_file() {
        return Err(format!(
            "Reflection extractor missing: {}. Build/distribute epok-header-tool alongside the editor.",
            tool.display()
        ));
    }
    let library = PathBuf::from(&config.libclang);
    let mut key = Sha256::new();
    key.update(&request_bytes);
    key.update(schema::SCHEMA_VERSION.to_le_bytes());
    key.update(schema::CLANG_VERSION);
    let mut tool_inputs = std::collections::BTreeMap::new();
    for path in [
        &tool,
        &compiler,
        &library.join(crate::dependencies::clang_library_name()),
        &sdk.join("common.mk"),
        &sdk.join("psyqo/psyqo.mk"),
    ] {
        let signature = hash_file(path)?;
        key.update(&signature);
        tool_inputs.insert(path.to_owned(), signature);
    }
    let key = format!("{:x}", key.finalize());
    let cache_path = folder.join("Reflection.epokcache");
    if let Ok(bytes) = fs::read(&cache_path)
        && let Ok(cache) = crate::document::from_slice::<Cache>(&bytes)
        && cache.key == key
        && cache
            .manifest
            .dependencies
            .iter()
            .all(|(path, hash)| hash_file(path).is_ok_and(|current| current == *hash))
    {
        crate::native_metadata::reflection(
            &root,
            &config,
            tool_inputs
                .into_iter()
                .chain(cache.manifest.dependencies.clone()),
        )?;
        return Ok(cache.manifest);
    }
    let mut command = Command::new(&tool);
    command
        .arg(&request_path)
        .current_dir(&root)
        .env("LIBCLANG_PATH", &library);
    crate::pipeline::quiet(&mut command);
    let output = command
        .output()
        .map_err(|e| format!("Reflection extractor: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "C++ reflection failed. Last cached metadata is stale and cannot be built.\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let manifest: Manifest =
        serde_json::from_slice(&output.stdout).map_err(|e| format!("Reflection metadata: {e}"))?;
    if manifest.schema_version != schema::SCHEMA_VERSION
        || manifest.clang_version != schema::CLANG_VERSION
    {
        return Err("Reflection extractor version mismatch".into());
    }
    if !manifest
        .dependencies
        .iter()
        .all(|(path, hash)| hash_file(path).is_ok_and(|current| current == *hash))
    {
        return Err(
            "A reflection dependency changed during extraction; retry after saving files".into(),
        );
    }
    let cache = Cache {
        key,
        manifest: manifest.clone(),
    };
    crate::native_metadata::reflection(
        &root,
        &config,
        tool_inputs.into_iter().chain(manifest.dependencies.clone()),
    )?;
    crate::settings::save_document(&cache_path, &cache)?;
    Ok(manifest)
}
