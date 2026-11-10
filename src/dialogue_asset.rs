//! Authoritative conversation/style documents. Scene receivers are bound to roles
//! at session start; a reusable conversation never stores a scene-object identity.
#![allow(dead_code)] // P3's compiler/runtime and P4's editor consume this schema.
use crate::assets::{self, Index, Kind, Record};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};
use uuid::Uuid;

pub const VERSION: u32 = 1;
pub const MAX_NODES: usize = 1024;
pub const MAX_PARTICIPANTS: usize = 8;
pub const MAX_BINDINGS: usize = 64;
pub const MAX_OPTIONS: usize = 64;
pub const VISIBLE_OPTIONS: usize = 4;
pub const MAX_IMMEDIATE_TRANSITIONS: usize = 32;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct TextKey {
    pub table: Uuid,
    pub key: String,
}
impl TextKey {
    fn validate(&self) -> Result<(), String> {
        if self.table.is_nil() || self.key.trim().is_empty() || self.key.len() > 128 {
            return Err(
                "Text needs a String Table UUID and a nonempty key of at most 128 bytes".into(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Participant {
    pub id: Uuid,
    pub label: String,
    pub display_name: Option<TextKey>,
    pub portrait: Option<Uuid>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BindingKind {
    Condition,
    Action,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub id: Uuid,
    pub label: String,
    pub participant: Uuid,
    pub kind: BindingKind,
    /// Reflected native/selected-provider class and member identities, never
    /// display names. The compiler checks the signature and emits typed glue.
    pub class: String,
    pub member: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct LinePolicy {
    /// Zero reveals the complete page immediately.
    pub milliseconds_per_glyph: u16,
    /// None waits for input. A timed line still yields to the next frame.
    pub auto_advance_ms: Option<u32>,
}
impl Default for LinePolicy {
    fn default() -> Self {
        Self {
            milliseconds_per_glyph: 30,
            auto_advance_ms: None,
        }
    }
}
impl LinePolicy {
    fn validate(&self) -> Result<(), String> {
        if self.milliseconds_per_glyph > 1000
            || self
                .auto_advance_ms
                .is_some_and(|n| n == 0 || n > 3_600_000)
        {
            return Err(
                "Reveal delay must be 0..1000 ms; automatic advance must be 1..3600000 ms".into(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ChoiceOption {
    pub id: Uuid,
    pub text: TextKey,
    pub next: Uuid,
    /// False options are omitted; display pagination is independent of count.
    pub condition: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NodeKind {
    Line {
        speaker: Option<Uuid>,
        text: TextKey,
        next: Uuid,
        #[serde(default)]
        policy: Option<LinePolicy>,
    },
    Choice {
        speaker: Option<Uuid>,
        prompt: Option<TextKey>,
        options: Vec<ChoiceOption>,
    },
    Branch {
        condition: Uuid,
        yes: Uuid,
        no: Uuid,
    },
    Event {
        action: Uuid,
        next: Uuid,
    },
    End,
}
impl NodeKind {
    pub fn destinations(&self) -> Vec<Uuid> {
        match self {
            Self::Line { next, .. } | Self::Event { next, .. } => vec![*next],
            Self::Choice { options, .. } => options.iter().map(|o| o.next).collect(),
            Self::Branch { yes, no, .. } => vec![*yes, *no],
            Self::End => Vec::new(),
        }
    }
    fn speaker(&self) -> Option<Uuid> {
        match self {
            Self::Line { speaker, .. } | Self::Choice { speaker, .. } => *speaker,
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: Uuid,
    pub kind: NodeKind,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DialogueAsset {
    pub version: u32,
    pub entry: Uuid,
    pub participants: Vec<Participant>,
    pub bindings: Vec<Binding>,
    pub nodes: Vec<Node>,
}
impl Default for DialogueAsset {
    fn default() -> Self {
        let entry = Uuid::new_v4();
        Self {
            version: VERSION,
            entry,
            participants: Vec::new(),
            bindings: Vec::new(),
            nodes: vec![Node {
                id: entry,
                kind: NodeKind::End,
            }],
        }
    }
}
impl DialogueAsset {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let value: Self =
            crate::document::from_slice(bytes).map_err(|e| format!("Dialogue: {e}"))?;
        value.validate()?;
        Ok(value)
    }
    pub fn texts(&self) -> Vec<&TextKey> {
        let mut result = self
            .participants
            .iter()
            .filter_map(|p| p.display_name.as_ref())
            .collect::<Vec<_>>();
        for node in &self.nodes {
            match &node.kind {
                NodeKind::Line { text, .. } => result.push(text),
                NodeKind::Choice {
                    prompt, options, ..
                } => {
                    result.extend(prompt);
                    result.extend(options.iter().map(|o| &o.text));
                }
                _ => {}
            }
        }
        result
    }
    pub fn reachable(&self) -> BTreeSet<Uuid> {
        let mut visited = BTreeSet::new();
        let mut pending = vec![self.entry];
        while let Some(id) = pending.pop() {
            if !visited.insert(id) {
                continue;
            }
            if let Some(node) = self.nodes.iter().find(|n| n.id == id) {
                pending.extend(node.kind.destinations());
            }
        }
        visited
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != VERSION
            || self.nodes.is_empty()
            || self.nodes.len() > MAX_NODES
            || self.participants.len() > MAX_PARTICIPANTS
            || self.bindings.len() > MAX_BINDINGS
        {
            return Err(format!(
                "Dialogue requires version {VERSION}, 1..{MAX_NODES} nodes, at most {MAX_PARTICIPANTS} participants and {MAX_BINDINGS} bindings"
            ));
        }
        let mut ids = BTreeSet::new();
        let mut unique = |id: Uuid| {
            if id.is_nil() || !ids.insert(id) {
                Err(format!("Dialogue has an empty or duplicate identity {id}"))
            } else {
                Ok(())
            }
        };
        for participant in &self.participants {
            unique(participant.id)?;
        }
        for binding in &self.bindings {
            unique(binding.id)?;
        }
        for node in &self.nodes {
            unique(node.id)?;
            if let NodeKind::Choice { options, .. } = &node.kind {
                for option in options {
                    unique(option.id)?;
                }
            }
        }
        let role = |id| self.participants.iter().any(|p| p.id == id);
        let binding = |id, kind| self.bindings.iter().any(|b| b.id == id && b.kind == kind);
        for value in &self.bindings {
            if !role(value.participant)
                || value.class.trim().is_empty()
                || value.member.trim().is_empty()
            {
                return Err(format!(
                    "Binding {} needs an existing participant and reflected class/member identities",
                    value.id
                ));
            }
        }
        let nodes = self.nodes.iter().map(|n| n.id).collect::<BTreeSet<_>>();
        if !nodes.contains(&self.entry) {
            return Err(format!("Dialogue entry {} is missing", self.entry));
        }
        for node in &self.nodes {
            for next in node.kind.destinations() {
                if !nodes.contains(&next) {
                    return Err(format!("Node {} has missing destination {next}", node.id));
                }
            }
            if node.kind.speaker().is_some_and(|id| !role(id)) {
                return Err(format!("Node {} has a missing speaker", node.id));
            }
            match &node.kind {
                NodeKind::Line {
                    policy: Some(policy),
                    ..
                } => policy
                    .validate()
                    .map_err(|e| format!("Node {}: {e}", node.id))?,
                NodeKind::Choice { options, .. } => {
                    if options.is_empty() || options.len() > MAX_OPTIONS {
                        return Err(format!("Node {} needs 1..{MAX_OPTIONS} options", node.id));
                    }
                    for option in options {
                        if option
                            .condition
                            .is_some_and(|id| !binding(id, BindingKind::Condition))
                        {
                            return Err(format!("Option {} needs a condition binding", option.id));
                        }
                    }
                }
                NodeKind::Branch { condition, .. }
                    if !binding(*condition, BindingKind::Condition) =>
                {
                    return Err(format!("Node {} needs a condition binding", node.id));
                }
                NodeKind::Event { action, .. } if !binding(*action, BindingKind::Action) => {
                    return Err(format!("Node {} needs an action binding", node.id));
                }
                _ => {}
            }
        }
        for text in self.texts() {
            text.validate()?;
        }
        if self
            .participants
            .iter()
            .any(|p| p.portrait.is_some_and(|id| id.is_nil()))
        {
            return Err("A portrait reference cannot be an empty UUID".into());
        }
        let reachable = self.reachable();
        if !self
            .nodes
            .iter()
            .any(|n| reachable.contains(&n.id) && n.kind == NodeKind::End)
        {
            return Err("Dialogue entry has no reachable End node".into());
        }
        Ok(())
    }
    pub fn validate_resources(&self, index: &Index) -> Result<(), String> {
        self.validate()?;
        for text in self.texts() {
            let record = index.resolve(text.table)?;
            let table = crate::localization::document(record)?;
            if table.key_index(&text.key).is_none() {
                return Err(format!(
                    "Dialogue String Table {} has no key '{}'",
                    text.table, text.key
                ));
            }
        }
        for participant in &self.participants {
            if let Some(id) = participant.portrait {
                require(index, id, Kind::Texture)?;
            }
        }
        Ok(())
    }
    /// Remaps internal identities while preserving class/member and asset IDs.
    pub fn duplicate(&self) -> Self {
        let mut map = std::collections::BTreeMap::new();
        for id in self
            .participants
            .iter()
            .map(|p| p.id)
            .chain(self.bindings.iter().map(|b| b.id))
            .chain(self.nodes.iter().map(|n| n.id))
            .chain(self.nodes.iter().flat_map(|n| match &n.kind {
                NodeKind::Choice { options, .. } => {
                    options.iter().map(|o| o.id).collect::<Vec<_>>()
                }
                _ => vec![],
            }))
        {
            map.insert(id, Uuid::new_v4());
        }
        let remap = |id: &mut Uuid| {
            if let Some(next) = map.get(id) {
                *id = *next
            }
        };
        let mut result = self.clone();
        remap(&mut result.entry);
        for role in &mut result.participants {
            remap(&mut role.id);
        }
        for binding in &mut result.bindings {
            remap(&mut binding.id);
            remap(&mut binding.participant);
        }
        for node in &mut result.nodes {
            remap(&mut node.id);
            match &mut node.kind {
                NodeKind::Line { speaker, next, .. } => {
                    if let Some(id) = speaker {
                        remap(id);
                    }
                    remap(next);
                }
                NodeKind::Choice {
                    speaker, options, ..
                } => {
                    if let Some(id) = speaker {
                        remap(id);
                    }
                    for option in options {
                        remap(&mut option.id);
                        remap(&mut option.next);
                        if let Some(id) = &mut option.condition {
                            remap(id);
                        }
                    }
                }
                NodeKind::Branch { condition, yes, no } => {
                    remap(condition);
                    remap(yes);
                    remap(no);
                }
                NodeKind::Event { action, next } => {
                    remap(action);
                    remap(next);
                }
                NodeKind::End => {}
            }
        }
        result
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Presentation {
    ClassicBox,
    PortraitConversation,
    Subtitles,
    SpeechBubble,
}
impl Presentation {
    pub const ALL: [Self; 4] = [
        Self::ClassicBox,
        Self::PortraitConversation,
        Self::Subtitles,
        Self::SpeechBubble,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::ClassicBox => "Classic Box",
            Self::PortraitConversation => "Portrait Conversation",
            Self::Subtitles => "Subtitles",
            Self::SpeechBubble => "Speech Bubble",
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Self::ClassicBox => "classic_box",
            Self::PortraitConversation => "portrait_conversation",
            Self::Subtitles => "subtitles",
            Self::SpeechBubble => "speech_bubble",
        }
    }
    pub fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.key() == key)
    }
}

pub const STYLE_VERSION: u32 = 2;

/// Session timing and sounds. Appearance belongs to the widget's controls.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DialogueStyle {
    pub version: u32,
    pub line_policy: LinePolicy,
    pub open_sound: Option<Uuid>,
    pub advance_sound: Option<Uuid>,
    pub close_sound: Option<Uuid>,
}
impl Default for DialogueStyle {
    fn default() -> Self {
        Self {
            version: STYLE_VERSION,
            line_policy: LinePolicy::default(),
            open_sound: None,
            advance_sound: None,
            close_sound: None,
        }
    }
}
impl DialogueStyle {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let document: serde_json::Value =
            crate::document::from_slice(bytes).map_err(|e| format!("Dialogue Style: {e}"))?;
        let value =
            match document.get("version").and_then(serde_json::Value::as_u64) {
                Some(1) => {
                    let old: LegacyStyleV1 = crate::document::from_slice(bytes)
                        .map_err(|e| format!("Dialogue Style v1: {e}"))?;
                    old.validate()?;
                    Self {
                        version: STYLE_VERSION,
                        line_policy: old.line_policy,
                        open_sound: old.open_sound,
                        advance_sound: old.advance_sound,
                        close_sound: old.close_sound,
                    }
                }
                Some(2) => crate::document::from_slice(bytes)
                    .map_err(|e| format!("Dialogue Style v2: {e}"))?,
                _ => return Err(
                    "Dialogue Style requires version 2 (version 1 supports read-only migration)"
                        .into(),
                ),
            };
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != STYLE_VERSION {
            return Err("Dialogue Style requires version 2".into());
        }
        if [self.open_sound, self.advance_sound, self.close_sound]
            .into_iter()
            .flatten()
            .any(|id| id.is_nil())
        {
            return Err("Dialogue Style sound references cannot be empty UUIDs".into());
        }
        self.line_policy.validate()
    }
    pub fn validate_resources(&self, index: &Index) -> Result<(), String> {
        self.validate()?;
        for id in [self.open_sound, self.advance_sound, self.close_sound]
            .into_iter()
            .flatten()
        {
            require(index, id, Kind::AudioClip)?;
        }
        Ok(())
    }
}

// Decode the previous document without reintroducing its visual configuration
// into either the current authoring model or the cooked runtime.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyStyleV1 {
    version: u32,
    presentation: Presentation,
    font: Option<Uuid>,
    panel: Option<Uuid>,
    panel_borders: [u16; 4],
    text_color: [f32; 3],
    panel_color: [f32; 3],
    margins: [u16; 4],
    portrait_size: [u16; 2],
    continuation_indicator: bool,
    line_policy: LinePolicy,
    open_sound: Option<Uuid>,
    advance_sound: Option<Uuid>,
    close_sound: Option<Uuid>,
}

impl LegacyStyleV1 {
    fn validate(&self) -> Result<(), String> {
        let _ = (self.presentation, self.continuation_indicator);
        if self.version != 1
            || self
                .text_color
                .iter()
                .chain(&self.panel_color)
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            || self
                .margins
                .iter()
                .chain(&self.panel_borders)
                .any(|n| *n > 1024)
            || self.portrait_size.iter().any(|n| *n == 0 || *n > 1024)
        {
            return Err("Invalid version 1 Dialogue Style visual data; preserve and repair the source before migration".into());
        }
        if [
            self.font,
            self.panel,
            self.open_sound,
            self.advance_sound,
            self.close_sound,
        ]
        .into_iter()
        .flatten()
        .any(|id| id.is_nil())
        {
            return Err("Dialogue Style v1 asset references cannot be empty UUIDs".into());
        }
        self.line_policy.validate()
    }
}
fn require(index: &Index, id: Uuid, kind: Kind) -> Result<(), String> {
    let record = index.resolve(id)?;
    if record.meta.kind != kind {
        return Err(format!(
            "Dialogue asset {id} is {:?}, expected {kind:?}",
            record.meta.kind
        ));
    }
    Ok(())
}
pub fn create(root: &Path, path: &str, dialogue: &DialogueAsset) -> Result<Uuid, String> {
    dialogue.validate()?;
    create_package(
        root,
        path,
        Kind::Dialogue,
        crate::document::to_vec(dialogue)?,
    )
}
pub fn create_style(root: &Path, path: &str, style: &DialogueStyle) -> Result<Uuid, String> {
    style.validate()?;
    create_package(
        root,
        path,
        Kind::DialogueStyle,
        crate::document::to_vec(style)?,
    )
}
fn create_package(root: &Path, path: &str, kind: Kind, source: Vec<u8>) -> Result<Uuid, String> {
    let id = Uuid::new_v4();
    let package = assets::Package {
        meta: assets::Metadata {
            version: 1,
            id,
            kind,
            importer_version: VERSION,
            source: path.into(),
            source_hash: assets::hash(&source),
            settings: crate::import_settings::Settings::Authored,
            extra: Default::default(),
        },
        source,
    };
    let destination = assets::inside(root, path)?;
    if destination.exists() {
        return Err(format!("Asset already exists: {path}"));
    }
    assets::atomic_write(&destination, &package.bytes()?, None)?;
    Ok(id)
}
pub fn document(record: &Record) -> Result<DialogueAsset, String> {
    if record.meta.kind != Kind::Dialogue {
        return Err("Select a Dialogue asset".into());
    }
    DialogueAsset::parse(&assets::Package::load(&record.path)?.source)
}
pub fn style_document(record: &Record) -> Result<DialogueStyle, String> {
    if record.meta.kind != Kind::DialogueStyle {
        return Err("Select a Dialogue Style asset".into());
    }
    DialogueStyle::parse(&assets::Package::load(&record.path)?.source)
}
pub fn save(record: &Record, dialogue: &DialogueAsset, index: &Index) -> Result<String, String> {
    dialogue.validate_resources(index)?;
    save_package(record, Kind::Dialogue, crate::document::to_vec(dialogue)?)
}
pub fn save_style(
    root: &Path,
    record: &Record,
    style: &DialogueStyle,
    index: &Index,
) -> Result<String, String> {
    style.validate_resources(index)?;
    let original = assets::read_bounded(&record.path)?;
    if assets::hash(&original) != record.revision {
        return Err("Dialogue Style changed on disk; reopen it before saving".into());
    }
    let package = assets::Package::parse(&original)?;
    if package.meta.id != record.meta.id || package.meta.kind != Kind::DialogueStyle {
        return Err("Dialogue Style asset was replaced; reopen it before saving".into());
    }
    let document: serde_json::Value = crate::document::from_slice(&package.source)?;
    if document.get("version").and_then(serde_json::Value::as_u64) == Some(1) {
        // Keep the complete old package outside assets, avoiding UUID conflicts.
        let canonical = root.canonicalize().map_err(|e| e.to_string())?;
        if !record
            .path
            .canonicalize()
            .map_err(|e| e.to_string())?
            .starts_with(&canonical)
        {
            return Err("Style migration must stay inside its project".into());
        }
        let folder = canonical
            .join(".epok/migrations/dialogue-style")
            .join(record.meta.id.to_string());
        std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
        if !folder
            .canonicalize()
            .map_err(|e| e.to_string())?
            .starts_with(&canonical)
        {
            return Err("Style migration backup resolves outside the project".into());
        }
        let backup = folder.join(format!("{}.epokasset", assets::hash(&original)));
        if backup.exists() {
            if assets::read_bounded(&backup)? != original {
                return Err("Style migration backup conflicts with the source".into());
            }
        } else {
            assets::atomic_write(&backup, &original, None)?;
        }
    }
    save_package(record, Kind::DialogueStyle, crate::document::to_vec(style)?)
}
fn save_package(record: &Record, kind: Kind, source: Vec<u8>) -> Result<String, String> {
    let mut package = assets::Package::load(&record.path)?;
    if package.meta.id != record.meta.id || package.meta.kind != kind {
        return Err("Dialogue asset was replaced; reopen it before saving".into());
    }
    package.source = source;
    package.meta.source_hash = assets::hash(&package.source);
    let bytes = package.bytes()?;
    assets::atomic_write(&record.path, &bytes, Some(&record.revision))?;
    Ok(assets::hash(&bytes))
}

#[cfg(test)]
#[path = "dialogue_style_migration_tests.rs"]
mod migration_tests;

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> DialogueAsset {
        let role = Uuid::new_v4();
        let condition = Uuid::new_v4();
        let action = Uuid::new_v4();
        let branch = Uuid::new_v4();
        let choice = Uuid::new_v4();
        let event = Uuid::new_v4();
        let end = Uuid::new_v4();
        let text = TextKey {
            table: Uuid::new_v4(),
            key: "dialogue.greeting".into(),
        };
        DialogueAsset {
            version: VERSION,
            entry: branch,
            participants: vec![Participant {
                id: role,
                label: "Speaker".into(),
                display_name: Some(text.clone()),
                portrait: None,
            }],
            bindings: vec![
                Binding {
                    id: condition,
                    label: "Available".into(),
                    participant: role,
                    kind: BindingKind::Condition,
                    class: "class-id".into(),
                    member: "condition-id".into(),
                },
                Binding {
                    id: action,
                    label: "Reward".into(),
                    participant: role,
                    kind: BindingKind::Action,
                    class: "class-id".into(),
                    member: "action-id".into(),
                },
            ],
            nodes: vec![
                Node {
                    id: branch,
                    kind: NodeKind::Branch {
                        condition,
                        yes: choice,
                        no: end,
                    },
                },
                Node {
                    id: choice,
                    kind: NodeKind::Choice {
                        speaker: Some(role),
                        prompt: Some(text.clone()),
                        options: (0..6)
                            .map(|_| ChoiceOption {
                                id: Uuid::new_v4(),
                                text: text.clone(),
                                next: event,
                                condition: Some(condition),
                            })
                            .collect(),
                    },
                },
                Node {
                    id: event,
                    kind: NodeKind::Event { action, next: end },
                },
                Node {
                    id: end,
                    kind: NodeKind::End,
                },
            ],
        }
    }
    #[test]
    fn dialogue_round_trip_and_duplication_preserve_external_resources() {
        let value = fixture();
        value.validate().unwrap();
        assert_eq!(
            DialogueAsset::parse(&crate::document::to_vec(&value).unwrap()).unwrap(),
            value
        );
        let copy = value.duplicate();
        copy.validate().unwrap();
        assert_ne!(copy.entry, value.entry);
        assert_ne!(copy.participants[0].id, value.participants[0].id);
        assert_eq!(copy.texts(), value.texts());
        assert_eq!(copy.bindings[0].member, value.bindings[0].member);
        assert!(copy.reachable().is_disjoint(&value.reachable()));
    }
    #[test]
    fn dialogue_rejects_dangling_ids_wrong_binding_kinds_and_invalid_timing() {
        let mut value = fixture();
        value.nodes[0].id = value.nodes[1].id;
        assert!(value.validate().unwrap_err().contains("duplicate identity"));
        let mut value = fixture();
        value.bindings[0].kind = BindingKind::Action;
        assert!(value.validate().unwrap_err().contains("condition binding"));
        let mut value = fixture();
        value.entry = Uuid::new_v4();
        assert!(value.validate().unwrap_err().contains("entry"));
        let mut value = fixture();
        value.nodes.pop();
        assert!(
            value
                .validate()
                .unwrap_err()
                .contains("missing destination")
        );
        assert!(
            LinePolicy {
                auto_advance_ms: Some(0),
                ..Default::default()
            }
            .validate()
            .is_err()
        );
        let style = DialogueStyle {
            open_sound: Some(Uuid::nil()),
            ..Default::default()
        };
        assert!(style.validate().is_err());
    }
    #[test]
    fn dialogue_packages_use_the_existing_asset_registry() {
        let root = crate::workspace::tests::temp("dialogue-assets");
        crate::workspace::create(&root, "Dialogue Assets", crate::workspace::Template::Basic)
            .unwrap();
        let id = create(
            &root,
            "assets/Conversation.epokasset",
            &DialogueAsset::default(),
        )
        .unwrap();
        let style_id = create_style(
            &root,
            "assets/ConversationStyle.epokasset",
            &DialogueStyle::default(),
        )
        .unwrap();
        assert!(
            create(
                &root,
                "assets/Conversation.epokasset",
                &DialogueAsset::default()
            )
            .is_err()
        );
        let index = assets::scan(&root, &mut Default::default());
        assert_eq!(document(index.resolve(id).unwrap()).unwrap().nodes.len(), 1);
        assert_eq!(
            style_document(index.resolve(style_id).unwrap()).unwrap(),
            DialogueStyle::default()
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn dialogue_resource_validation_and_atomic_saves_preserve_identity() {
        let root = crate::workspace::tests::temp("dialogue-resources");
        crate::workspace::create(
            &root,
            "Dialogue Resources",
            crate::workspace::Template::Basic,
        )
        .unwrap();
        let table = crate::localization::create(
            &root,
            "assets/Strings.epokasset",
            &crate::localization::StringTable {
                languages: vec!["English".into()],
                entries: vec![crate::localization::Entry {
                    key: "dialogue.hello".into(),
                    values: vec!["Hello".into()],
                }],
                ..Default::default()
            },
        )
        .unwrap();
        let mut dialogue = DialogueAsset::default();
        let end = dialogue.entry;
        dialogue.entry = Uuid::new_v4();
        dialogue.nodes.push(Node {
            id: dialogue.entry,
            kind: NodeKind::Line {
                speaker: None,
                text: TextKey {
                    table,
                    key: "dialogue.hello".into(),
                },
                next: end,
                policy: None,
            },
        });
        let id = create(&root, "assets/Hello.epokasset", &dialogue).unwrap();
        let style_id =
            create_style(&root, "assets/Style.epokasset", &DialogueStyle::default()).unwrap();
        let index = assets::scan(&root, &mut Default::default());
        dialogue.validate_resources(&index).unwrap();
        if let NodeKind::Line { text, .. } = &mut dialogue.nodes[1].kind {
            text.key = "missing".into();
        }
        assert!(
            dialogue
                .validate_resources(&index)
                .unwrap_err()
                .contains("no key 'missing'")
        );
        if let NodeKind::Line { text, policy, .. } = &mut dialogue.nodes[1].kind {
            text.key = "dialogue.hello".into();
            *policy = Some(LinePolicy {
                auto_advance_ms: Some(100),
                ..Default::default()
            });
        }
        let record = index.resolve(id).unwrap();
        let revision = save(record, &dialogue, &index).unwrap();
        assert_ne!(revision, record.revision);
        assert_eq!(document(record).unwrap(), dialogue);
        assert_eq!(assets::Package::load(&record.path).unwrap().meta.id, id);
        assert!(
            save(record, &dialogue, &index).is_err(),
            "stale revision cannot overwrite a later save"
        );
        let mut style = DialogueStyle {
            open_sound: Some(table),
            ..Default::default()
        };
        assert!(
            style
                .validate_resources(&index)
                .unwrap_err()
                .contains("expected AudioClip")
        );
        style.open_sound = None;
        style.line_policy.milliseconds_per_glyph = 12;
        let record = index.resolve(style_id).unwrap();
        save_style(&root, record, &style, &index).unwrap();
        assert_eq!(style_document(record).unwrap(), style);
        std::fs::remove_dir_all(root).unwrap();
    }
}
