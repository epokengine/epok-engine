//! Host artifact provenance, using existing compiler IDs and content signatures.
//! This graph never resolves assets, reflection members or runtime handles.
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::Path,
    sync::Mutex,
};

const VERSION: u32 = 1;
const FILE: &str = ".epok/ArtifactDependencies.json";
// Editor validation and the build worker can publish in the same process.
static TRANSACTION: Mutex<()> = Mutex::new(());
thread_local! {
    // Compare actual bytes on every read: external edits, including same-size
    // writes with preserved timestamps, must invalidate this parsed snapshot.
    static LAST_READ: std::cell::RefCell<Option<(std::path::PathBuf, Vec<u8>, Graph)>> = const { std::cell::RefCell::new(None) };
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    /// Last successfully observed or generated content. Stale is never usable.
    pub signature: Option<String>,
    pub dependencies: BTreeSet<String>,
    pub stale: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Graph {
    version: u32,
    pub nodes: BTreeMap<String, Node>,
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            version: VERSION,
            nodes: BTreeMap::new(),
        }
    }
}

impl Graph {
    /// Follow existing provenance edges, including retained stale inputs.
    pub fn depends_on_any(&self, root: &str, inputs: &BTreeSet<String>) -> bool {
        let mut pending = vec![root];
        let mut visited = BTreeSet::new();
        while let Some(key) = pending.pop() {
            if !visited.insert(key) {
                continue;
            }
            if inputs.contains(key) {
                return true;
            }
            if let Some(node) = self.nodes.get(key) {
                pending.extend(node.dependencies.iter().map(String::as_str));
            }
        }
        false
    }
    pub fn load(root: &Path) -> Result<Self, String> {
        let path = root.join(FILE);
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => return Err(format!("Artifact dependency graph: {error}")),
        };
        Self::decode(&path, &bytes)
    }

    fn decode(path: &Path, bytes: &[u8]) -> Result<Self, String> {
        if let Some(graph) = LAST_READ.with_borrow(|last| {
            last.as_ref()
                .filter(|(previous_path, previous_bytes, _)| {
                    previous_path == path && previous_bytes == bytes
                })
                .map(|(_, _, graph)| graph.clone())
        }) {
            return Ok(graph);
        }
        let graph: Self = serde_json::from_slice(bytes)
            .map_err(|error| format!("Artifact dependency graph: {error}"))?;
        if graph.version != VERSION {
            return Err(format!(
                "Unsupported artifact dependency graph version {}",
                graph.version
            ));
        }
        LAST_READ
            .with_borrow_mut(|last| *last = Some((path.to_owned(), bytes.to_vec(), graph.clone())));
        Ok(graph)
    }

    /// Fresh input observation or successful artifact generation. A successful
    /// dependency rebuild does not certify its consumers: they must republish.
    pub fn publish(&mut self, id: &str, signature: String, dependencies: BTreeSet<String>) {
        let changed = self.nodes.get(id).is_none_or(|old| {
            old.signature.as_ref() != Some(&signature)
                || old.dependencies != dependencies
                || !old.stale.is_empty()
        });
        if changed {
            self.invalidate(id, "Dependency content or validity changed");
        }
        let mut stale = BTreeMap::new();
        for dependency in &dependencies {
            match self.nodes.get(dependency) {
                Some(node) if node.signature.is_some() && node.stale.is_empty() => {}
                _ => {
                    stale.insert(
                        dependency.clone(),
                        "Dependency has no fresh artifact".into(),
                    );
                }
            }
        }
        self.nodes.insert(
            id.into(),
            Node {
                signature: Some(signature),
                dependencies,
                stale,
            },
        );
    }

    /// Keep old reverse edges and last valid signatures when a source disappears
    /// or fails to parse. They are essential to invalidate former consumers.
    pub fn invalidate(&mut self, id: &str, reason: &str) {
        self.nodes.entry(id.into()).or_default();
        let mut reverse: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (consumer, node) in &self.nodes {
            for dependency in &node.dependencies {
                reverse
                    .entry(dependency.clone())
                    .or_default()
                    .push(consumer.clone());
            }
        }
        let mut queue = VecDeque::from([id.to_owned()]);
        let mut visited = BTreeSet::new();
        while let Some(next) = queue.pop_front() {
            if !visited.insert(next.clone()) {
                continue;
            }
            self.nodes
                .get_mut(&next)
                .expect("Known graph node")
                .stale
                .insert(id.into(), reason.into());
            if let Some(consumers) = reverse.get(&next) {
                queue.extend(consumers.iter().cloned());
            }
        }
    }
}

pub fn transaction(root: &Path, update: impl FnOnce(&mut Graph)) -> Result<Graph, String> {
    let _guard = TRANSACTION
        .lock()
        .map_err(|_| "Artifact dependency transaction failed")?;
    let path = root.join(FILE);
    let before = match std::fs::read(&path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(format!("Artifact dependency graph: {error}")),
    };
    let mut graph = match &before {
        Some(bytes) => Graph::decode(&path, bytes)?,
        None => Graph::default(),
    };
    let previous = graph.clone();
    update(&mut graph);
    if before.is_some() && graph == previous {
        return Ok(graph);
    }
    let bytes = serde_json::to_vec_pretty(&graph).map_err(|e| e.to_string())?;
    if before.as_deref() != Some(bytes.as_slice()) {
        let revision = before.as_deref().map(crate::assets::hash);
        crate::assets::atomic_write(&path, &bytes, revision.as_deref())?;
    }
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unchanged_transactions_preserve_disk_bytes_and_same_stamp_edits_are_observed() {
        let root = std::env::temp_dir().join(format!("epok-graph-read-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join(".epok")).unwrap();
        let path = root.join(FILE);
        let mut graph = Graph::default();
        graph.publish("source", "a".into(), Default::default());
        let compact = serde_json::to_vec(&graph).unwrap();
        std::fs::write(&path, &compact).unwrap();
        let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
        assert_eq!(Graph::load(&root).unwrap(), graph);
        transaction(&root, |graph| {
            graph.publish("source", "a".into(), Default::default())
        })
        .unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), compact);
        assert_eq!(
            std::fs::metadata(&path).unwrap().modified().unwrap(),
            modified
        );

        graph.publish("source", "b".into(), Default::default());
        let edited = serde_json::to_vec(&graph).unwrap();
        assert_eq!(edited.len(), compact.len());
        std::fs::write(&path, &edited).unwrap();
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(modified))
            .unwrap();
        assert_eq!(Graph::load(&root).unwrap(), graph);
        let updated = transaction(&root, |graph| {
            assert_eq!(graph.nodes["source"].signature.as_deref(), Some("b"));
            graph.publish("other", "c".into(), Default::default());
        })
        .unwrap();
        assert_eq!(Graph::load(&root).unwrap(), updated);
        std::fs::write(&path, b"invalid").unwrap();
        assert!(Graph::load(&root).is_err());
        std::fs::remove_file(&path).unwrap();
        assert_eq!(Graph::load(&root).unwrap(), Graph::default());
        std::fs::remove_dir(root.join(".epok")).unwrap();
        std::fs::remove_dir(root).unwrap();
    }

    fn publish(graph: &mut Graph, id: &str, dependencies: &[&str]) {
        graph.publish(
            id,
            "1".into(),
            dependencies.iter().map(|s| s.to_string()).collect(),
        );
    }
    #[test]
    fn changes_and_deletions_follow_old_edges_without_certifying_consumers_on_recovery() {
        let mut graph = Graph::default();
        publish(&mut graph, "property:caster:charge", &[]);
        publish(&mut graph, "timeline:cast", &["property:caster:charge"]);
        publish(&mut graph, "effect:fireball", &["timeline:cast"]);
        publish(&mut graph, "scene:combat", &["effect:fireball"]);
        publish(&mut graph, "stage:game", &["scene:combat"]);
        publish(&mut graph, "export:game", &["stage:game"]);
        publish(&mut graph, "timeline:door", &[]);
        graph.invalidate("property:caster:charge", "Property removed");
        for id in [
            "timeline:cast",
            "effect:fireball",
            "scene:combat",
            "stage:game",
            "export:game",
        ] {
            assert_eq!(
                graph.nodes[id].stale["property:caster:charge"],
                "Property removed"
            );
            assert_eq!(graph.nodes[id].signature.as_deref(), Some("1"));
        }
        assert!(graph.nodes["timeline:door"].stale.is_empty());
        publish(&mut graph, "property:caster:charge", &[]);
        assert!(!graph.nodes["timeline:cast"].stale.is_empty());
        publish(&mut graph, "timeline:cast", &["property:caster:charge"]);
        assert!(graph.nodes["timeline:cast"].stale.is_empty());
        assert!(!graph.nodes["export:game"].stale.is_empty());
        // After an intentional rewire, changing the former dependency no longer
        // invalidates this output. Old edges survive errors, not successful edits.
        publish(&mut graph, "timeline:cast", &[]);
        graph.invalidate("property:caster:charge", "Removed again");
        assert!(graph.nodes["timeline:cast"].stale.is_empty());
    }
    #[test]
    fn cyclic_invalid_input_terminates_and_missing_dependencies_cannot_be_fresh() {
        let mut graph = Graph::default();
        publish(&mut graph, "a", &["b"]);
        publish(&mut graph, "b", &["a"]);
        graph.invalidate("a", "Invalid cycle");
        assert_eq!(graph.nodes["b"].stale["a"], "Invalid cycle");
        assert!(!graph.nodes["a"].stale.is_empty());
    }
}
