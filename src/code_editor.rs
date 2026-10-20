//! Arguments for the external source editor, including Windows path interoperability.
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub fn command(executable: &Path, root: &Path, source: &Path, line: Option<usize>) -> Command {
    let mut command = Command::new(executable);
    command.arg("--reuse-window").arg(editor_path(root));
    let source = editor_path(source);
    if let Some(line) = line {
        command
            .arg("--goto")
            .arg(format!("{}:{}", source.display(), line.max(1)));
    } else {
        command.arg(source);
    }
    command
}

/// Rust's canonical paths use Win32 verbatim prefixes. VS Code cannot open
/// those workspace/file arguments reliably; keep canonical paths internally,
/// but pass ordinary drive/UNC paths across this application boundary.
pub(crate) fn editor_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        use std::path::{Component, Prefix};
        let mut components = path.components();
        if let Some(Component::Prefix(prefix)) = components.next() {
            let mut normal = match prefix.kind() {
                Prefix::VerbatimDisk(drive) => PathBuf::from(format!("{}:", char::from(drive))),
                Prefix::VerbatimUNC(server, share) => {
                    let mut prefix = std::ffi::OsString::from("\\\\");
                    prefix.push(server);
                    prefix.push("\\");
                    prefix.push(share);
                    PathBuf::from(prefix)
                }
                _ => return path.to_path_buf(),
            };
            normal.extend(components);
            return normal;
        }
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn canonical_lua_and_cpp_sources_open_in_their_workspace_at_the_requested_line() {
        let root = Path::new(r"\\?\C:\Users\Author\Epok Projects\Dungeon");
        for source in ["DungeonKey.lua", "DungeonDoor.hpp"] {
            let command = command(
                Path::new(r"C:\Program Files\Microsoft VS Code\Code.exe"),
                root,
                &root.join("assets/scripts").join(source),
                Some(2),
            );
            let args: Vec<_> = command
                .get_args()
                .map(|v| v.to_string_lossy().into_owned())
                .collect();
            assert_eq!(
                args,
                [
                    "--reuse-window".into(),
                    r"C:\Users\Author\Epok Projects\Dungeon".into(),
                    "--goto".into(),
                    format!(r"C:\Users\Author\Epok Projects\Dungeon\assets\scripts\{source}:2"),
                ]
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn network_sources_keep_the_server_and_share() {
        assert_eq!(
            editor_path(Path::new(r"\\?\UNC\server\games\Dungeon\Guard.lua")),
            Path::new(r"\\server\games\Dungeon\Guard.lua")
        );
        assert_eq!(
            editor_path(Path::new(r"\\server\games\Dungeon\Guard.lua")),
            Path::new(r"\\server\games\Dungeon\Guard.lua")
        );
    }

    #[test]
    fn open_folder_and_unpositioned_source_do_not_add_goto() {
        let root = Path::new("/projects/Dungeon");
        let source = root.join("assets/scripts");
        let command = command(Path::new("code"), root, &source, None);
        let args: Vec<_> = command.get_args().collect();
        assert_eq!(
            args,
            [
                std::ffi::OsStr::new("--reuse-window"),
                root.as_os_str(),
                source.as_os_str()
            ]
        );
    }
}
