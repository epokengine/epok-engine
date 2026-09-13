//! Isolated host process: libclang owns its AST and is never loaded into the editor.
mod document;
use clang::{Clang, Index};
use std::fs;
mod header_extract;
mod reflection_schema;
use reflection_schema::Request;

fn run() -> Result<(), String> {
    let input = std::env::args_os()
        .nth(1)
        .ok_or("Usage: epok-header-tool <Reflection.epokrequest>")?;
    let request: Request = document::from_slice(&fs::read(input).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let clang = Clang::new().map_err(|e| {
        format!("Host reflection needs pinned libclang 18.1.1. Run tools/setup.ps1: {e}")
    })?;
    let version = clang::get_version();
    if !version.contains("18.1.1") {
        return Err(format!("Expected libclang 18.1.1; found {version}"));
    }
    let index = Index::new(&clang, false, false);
    let unit = index
        .parser(&request.source)
        .arguments(&request.arguments)
        .detailed_preprocessing_record(true)
        .parse()
        .map_err(|e| format!("Clang parse: {e:?}"))?;
    let errors = unit
        .get_diagnostics()
        .into_iter()
        .filter(|d| d.get_severity() >= clang::diagnostic::Severity::Error)
        .map(|d| d.to_string())
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    let manifest = header_extract::extract(&unit, &request.source)?;
    let mut validation = fs::read_to_string(&request.source).map_err(|e| e.to_string())?;
    for class in manifest
        .classes
        .iter()
        .filter(|class| !class.abstract_class)
    {
        validation.push_str(&format!("\nstatic_assert(__is_constructible({0}), \"Reflected class must be publicly default-constructible\");\nstatic_assert(__is_assignable({0}&, {0}), \"Reflected class must support scene-bank reset assignment\");\n",class.cpp_name));
    }
    let unsaved = clang::Unsaved::new(&request.source, &validation);
    let validated = index
        .parser(&request.source)
        .arguments(&request.arguments)
        .unsaved(&[unsaved])
        .parse()
        .map_err(|e| format!("Class eligibility validation: {e:?}"))?;
    let errors = validated
        .get_diagnostics()
        .into_iter()
        .filter(|d| d.get_severity() >= clang::diagnostic::Severity::Error)
        .map(|d| d.to_string())
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
