//! Disposable editor artifacts, never authored truth. Every lookup hashes the
//! actual source bytes (not just mtime/size), including when the asset index is
//! stale. A bounded process cache shares immutable decoded resources across maps;
//! a checksummed disk cache avoids repeating import work after editor restart.
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    any::Any,
    collections::VecDeque,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
};

const MAGIC: &[u8; 8] = b"EPOKC01\n";
const LIMIT: usize = 256 * 1024 * 1024;
const MEMORY_BUDGET: usize = 128 * 1024 * 1024;
struct Entry {
    key: (PathBuf, String),
    hash: [u8; 32],
    weight: usize,
    value: Arc<dyn Any + Send + Sync>,
}
#[derive(Default)]
struct Memory {
    entries: VecDeque<Entry>,
    bytes: usize,
}
fn memory() -> &'static Mutex<Memory> {
    static CACHE: OnceLock<Mutex<Memory>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}
fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
fn read(path: &Path) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if file.metadata().map_err(|e| e.to_string())?.len() > LIMIT as u64 {
        return Err(format!(
            "{} exceeds editor artifact size limit",
            path.display()
        ));
    }
    let mut bytes = Vec::new();
    file.take(LIMIT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > LIMIT {
        return Err("Editor artifact size limit exceeded".into());
    }
    Ok(bytes)
}
fn artifact(path: &Path, kind: &str) -> Option<PathBuf> {
    let root = path.ancestors().find(|p| p.join("assets").is_dir())?;
    let id = digest(path.strip_prefix(root).ok()?.to_string_lossy().as_bytes());
    let id: String = id.iter().map(|b| format!("{b:02x}")).collect();
    Some(
        root.join(".epok/editor-cache/v1")
            .join(format!("{kind}-{id}.cache")),
    )
}
fn store(path: &Path, source: [u8; 32], payload: &[u8]) -> std::io::Result<()> {
    fs::create_dir_all(path.parent().unwrap())?;
    let temp = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        file.write_all(MAGIC)?;
        file.write_all(&source)?;
        file.write_all(&digest(payload))?;
        file.write_all(payload)?;
        drop(file);
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

/// `kind` is a versioned importer/schema identity. Increment it when changing
/// derived semantics. Decoders must depend only on these bytes and that identity;
/// dependency-dependent results need their own dependency key, not this cache.
pub fn load<T>(
    path: &Path,
    kind: &str,
    decode: impl FnOnce(&[u8]) -> Result<T, String>,
) -> Result<Arc<T>, String>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    // The artifact bound must not become a new limit on authored scene sizes.
    // Oversized inputs use the original decoding path without caching.
    // Preserve the package reader's header/size checks before allocating its
    // payload. Scene documents keep their original unrestricted source reader.
    let bytes = if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("epokasset"))
    {
        crate::assets::read_bounded(path)?
    } else {
        fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?
    };
    if bytes.len() > LIMIT {
        return decode(&bytes).map(Arc::new);
    }
    let path = path.canonicalize().map_err(|e| e.to_string())?;
    let hash = digest(&bytes);
    let key = (path.clone(), kind.to_owned());
    {
        let mut cache = memory().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(i) = cache.entries.iter().position(|e| e.key == key) {
            let entry = cache.entries.remove(i).unwrap();
            cache.bytes -= entry.weight;
            if entry.hash == hash {
                if let Ok(value) = entry.value.clone().downcast::<T>() {
                    cache.bytes += entry.weight;
                    cache.entries.push_back(entry);
                    return Ok(value);
                }
            }
        }
    }
    let disk = artifact(&path, kind);
    let cached = disk.as_deref().and_then(|p| read(p).ok()).and_then(|data| {
        if data.len() < 72
            || &data[..8] != MAGIC
            || data[8..40] != hash
            || data[40..72] != digest(&data[72..])
        {
            return None;
        }
        serde_json::from_slice::<T>(&data[72..])
            .ok()
            .map(|value| (value, data.len()))
    });
    let (value, size) = if let Some(cached) = cached {
        cached
    } else {
        let value = decode(&bytes)?;
        // Cache I/O is an optimization: unwritable/deleted caches cannot prevent
        // opening a valid scene or publishing the decoded source.
        let payload = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
        if payload.len() <= LIMIT - 72 {
            if let Some(path) = &disk {
                let _ = store(path, hash, &payload);
            }
        }
        (value, payload.len())
    };
    let value = Arc::new(value);
    // Conservative estimate for containers/strings; never keep an unbounded
    // history of scenes or revisions. Live scenes may independently retain Arcs.
    // Source bytes are temporary, not retained by the cache. Charging a tiny
    // metadata entry for the size of its entire map would evict the useful
    // decoded resources whenever a large visibility track is indexed.
    let weight = size.saturating_mul(4).max(256);
    if weight <= MEMORY_BUDGET {
        let mut cache = memory().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(i) = cache.entries.iter().position(|e| e.key == key) {
            let old = cache.entries.remove(i).unwrap();
            cache.bytes -= old.weight;
        }
        while cache.bytes + weight > MEMORY_BUDGET || cache.entries.len() >= 2048 {
            let Some(old) = cache.entries.pop_front() else {
                break;
            };
            cache.bytes -= old.weight;
        }
        cache.entries.push_back(Entry {
            key,
            hash,
            weight,
            value: value.clone(),
        });
        cache.bytes += weight;
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> PathBuf {
        let root = std::env::temp_dir().join(format!("epok-source-cache-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("assets")).unwrap();
        root.join("assets/source.txt")
    }
    fn forget(path: &Path) {
        let path = path.canonicalize().unwrap();
        let mut cache = memory().lock().unwrap();
        cache.entries.retain(|e| e.key.0 != path);
        cache.bytes = cache.entries.iter().map(|e| e.weight).sum();
    }
    fn decode(bytes: &[u8]) -> Result<String, String> {
        String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())
    }
    #[test]
    fn reuse_restart_external_edit_and_deletion() {
        let path = fixture();
        fs::write(&path, "first").unwrap();
        let a = load(&path, "test-v1", decode).unwrap();
        let b = load::<String>(&path, "test-v1", |_| panic!("memory miss")).unwrap();
        assert!(Arc::ptr_eq(&a, &b));
        forget(&path);
        assert_eq!(
            *load::<String>(&path, "test-v1", |_| panic!("disk miss")).unwrap(),
            "first"
        );
        let modified = fs::metadata(&path).unwrap().modified().unwrap();
        fs::write(&path, "other").unwrap();
        fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(modified))
            .unwrap();
        assert_eq!(*load(&path, "test-v1", decode).unwrap(), "other");
        forget(&path);
        assert_eq!(
            *load::<String>(&path, "test-v1", |_| panic!(
                "updated disk artifact was not replaced"
            ))
            .unwrap(),
            "other"
        );
        fs::remove_file(&path).unwrap();
        assert!(load(&path, "test-v1", decode).is_err());
        fs::remove_dir_all(path.parent().unwrap().parent().unwrap()).unwrap();
    }
    #[test]
    fn corrupt_cache_and_importer_changes_regenerate_from_source() {
        let path = fixture();
        fs::write(&path, "valid").unwrap();
        load(&path, "test-v1", decode).unwrap();
        forget(&path);
        fs::write(
            artifact(&path.canonicalize().unwrap(), "test-v1").unwrap(),
            "broken",
        )
        .unwrap();
        assert_eq!(*load(&path, "test-v1", decode).unwrap(), "valid");
        assert_eq!(
            *load(&path, "test-v2", |_| Ok(String::from("v2"))).unwrap(),
            "v2"
        );
        forget(&path);
        fs::remove_dir_all(path.parent().unwrap().parent().unwrap()).unwrap();
    }
    #[test]
    fn unavailable_cache_directory_does_not_block_valid_source() {
        let path = fixture();
        let root = path.parent().unwrap().parent().unwrap();
        fs::write(&path, "source").unwrap();
        fs::create_dir_all(root.join(".epok")).unwrap();
        fs::write(root.join(".epok/editor-cache"), "not a directory").unwrap();
        assert_eq!(*load(&path, "test-v1", decode).unwrap(), "source");
        forget(&path);
        assert_eq!(*load(&path, "test-v1", decode).unwrap(), "source");
        forget(&path);
        fs::remove_dir_all(root).unwrap();
    }
}
