//! Named, typed asset references. UUIDs survive moves; keys are scoped to a table.
use crate::assets::{self, Index, Kind, Record};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};
use uuid::Uuid;

pub const MAX_ROWS: usize = 1024;
pub const KINDS: &[&str] = &[
    "Asset",
    "AudioClip",
    "Texture",
    "MusicSequence",
    "Video",
    "PocketStationApp",
    "DataTable",
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub key: String,
    pub kind: Kind,
    pub asset: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DataTable {
    pub version: u32,
    /// None permits mixed kinds; Some constrains every row to that kind.
    pub row_type: Option<Kind>,
    pub rows: Vec<Row>,
}
impl Default for DataTable {
    fn default() -> Self {
        Self {
            version: 1,
            row_type: Some(Kind::AudioClip),
            rows: Vec::new(),
        }
    }
}
impl DataTable {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let table: Self =
            crate::document::from_slice(bytes).map_err(|e| format!("DataTable: {e}"))?;
        table.validate()?;
        Ok(table)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.rows.len() > MAX_ROWS {
            return Err(format!(
                "DataTable requires version 1 and at most {MAX_ROWS} rows"
            ));
        }
        let mut keys = BTreeSet::new();
        for row in &self.rows {
            validate_key(&row.key)?;
            if !keys.insert(&row.key) {
                return Err(format!("Duplicate DataTable key '{}'", row.key));
            }
            if row.asset.is_nil() {
                return Err(format!("DataTable row '{}' has no asset", row.key));
            }
            if self.row_type.as_ref().is_some_and(|kind| kind != &row.kind) {
                return Err(format!(
                    "DataTable row '{}' must have type {:?}",
                    row.key,
                    self.row_type.as_ref().unwrap()
                ));
            }
        }
        Ok(())
    }
    pub fn validate_assets(&self, index: &Index) -> Result<(), String> {
        self.validate()?;
        for row in &self.rows {
            let record = index
                .resolve(row.asset)
                .map_err(|e| format!("DataTable row '{}': {e}", row.key))?;
            // Tables use exact kinds, including AudioClip versus MusicSequence.
            if record.meta.kind != row.kind {
                return Err(format!(
                    "DataTable row '{}' references {:?}, expected {:?}",
                    row.key, record.meta.kind, row.kind
                ));
            }
        }
        Ok(())
    }
    #[cfg(test)]
    pub fn find(&self, key: &str, kind: Option<&Kind>) -> Option<Uuid> {
        self.rows
            .iter()
            .find(|row| row.key == key && kind.is_none_or(|kind| kind.accepts_runtime(&row.kind)))
            .map(|row| row.asset)
    }
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        serde_json::to_vec_pretty(self).map_err(|e| e.to_string())
    }
}
pub fn validate_key(key: &str) -> Result<(), String> {
    if key.is_empty() || key.len() > 128 || key.trim() != key || key.chars().any(char::is_control) {
        return Err("DataTable keys require 1..128 UTF-8 bytes, no control characters or surrounding whitespace".into());
    }
    Ok(())
}
pub fn kind_code(kind: &str) -> Result<u32, String> {
    match kind {
        "Asset" => Ok(0),
        "Texture" => Ok(1),
        "AudioClip" | "PlayableAudio" => Ok(2),
        "Video" => Ok(3),
        "PocketStationApp" => Ok(4),
        "DataTable" => Ok(5),
        "MusicSequence" => Ok(6),
        _ => Err(format!(
            "Runtime DataTable references do not support {kind}"
        )),
    }
}
pub fn document(record: &Record) -> Result<DataTable, String> {
    if record.meta.kind != Kind::DataTable {
        return Err("Select a DataTable asset".into());
    }
    DataTable::parse(&assets::Package::load(&record.path)?.source)
}
pub fn dependencies(index: &Index, id: Uuid) -> Result<BTreeSet<Uuid>, String> {
    let mut found = BTreeSet::new();
    let mut pending = vec![id];
    while let Some(id) = pending.pop() {
        if !found.insert(id) {
            continue;
        }
        let record = index.resolve(id)?;
        if record.meta.kind == Kind::DataTable {
            let table = document(record)?;
            table.validate_assets(index)?;
            pending.extend(table.rows.iter().map(|row| row.asset));
        }
    }
    Ok(found)
}
pub fn create(root: &Path, path: &str, table: &DataTable) -> Result<Uuid, String> {
    let id = Uuid::new_v4();
    let source = table.encode()?;
    let package = assets::Package {
        meta: assets::Metadata {
            version: 2,
            id,
            kind: Kind::DataTable,
            importer_version: 1,
            source: path.into(),
            source_hash: assets::hash(&source),
            settings: crate::import_settings::Settings::Authored,
            extra: Default::default(),
        },
        source,
    };
    assets::atomic_write(&assets::inside(root, path)?, &package.bytes()?, None)?;
    Ok(id)
}
pub fn save(record: &Record, table: &DataTable, index: &Index) -> Result<String, String> {
    table.validate_assets(index)?;
    let mut package = assets::Package::load(&record.path)?;
    if package.meta.id != record.meta.id || package.meta.kind != Kind::DataTable {
        return Err("DataTable was replaced; reopen it before saving".into());
    }
    package.source = table.encode()?;
    package.meta.source_hash = assets::hash(&package.source);
    let bytes = package.bytes()?;
    assets::atomic_write(&record.path, &bytes, Some(&record.revision))?;
    Ok(assets::hash(&bytes))
}

/// Emitted with the scene's shared resources, so native and script lookups agree.
pub fn header(tables: &std::collections::BTreeMap<Uuid, DataTable>) -> Result<String, String> {
    let mut rows = Vec::new();
    let mut compact = std::collections::BTreeMap::new();
    for (id, table) in tables {
        let table_id = crate::blueprint_refs::compact_id(&id.to_string());
        if table_id == 0 {
            return Err("Invalid compact DataTable identity".into());
        }
        if let Some(previous) = compact.insert(table_id, *id)
            && previous != *id
        {
            return Err("Compact DataTable identity collision".into());
        }
        for row in &table.rows {
            let asset = crate::blueprint_refs::compact_id(&row.asset.to_string());
            let kind = kind_code(&format!("{:?}", row.kind))?;
            if let Some(previous) = compact.insert(asset, row.asset)
                && previous != row.asset
            {
                return Err("Compact DataTable asset identity collision".into());
            }
            rows.push(format!(
                "{{UINT64_C({table_id}),{},UINT64_C({asset}),{kind}}}",
                serde_json::to_string(&row.key).unwrap()
            ));
        }
    }
    let count = rows.len();
    if rows.is_empty() {
        rows.push("{}".into());
    }
    Ok(format!(
        "namespace epok::bp {{\nstruct DataTableRow {{uint64_t table;const char* key;uint64_t asset;uint32_t kind;}};\ninline const DataTableRow data_table_rows[]={{{}}};\ninline bool data_table_kind(uint32_t actual,uint32_t expected){{return !expected||actual==expected||(expected==2&&actual==6);}}\ninline bool data_table_key(const char* a,const char* b){{if(!a||!b)return false;while(*a&&*a==*b){{++a;++b;}}return *a==*b;}}\ninline uint64_t data_table_find(uint64_t table,const char* key,uint32_t kind){{for(size_t i=0;i<{count};++i){{const auto& row=data_table_rows[i];if(row.table==table&&data_table_key(row.key,key)&&data_table_kind(row.kind,kind))return row.asset;}}return 0;}}\ninline uint64_t data_table_cast(uint64_t asset,uint32_t kind){{for(size_t i=0;i<{count};++i)if(data_table_rows[i].asset==asset&&data_table_kind(data_table_rows[i].kind,kind))return asset;return 0;}}\n}}\n",
        rows.join(",")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keys_types_and_checked_lookup() {
        let id = Uuid::new_v4();
        let mut table = DataTable {
            rows: vec![Row {
                key: "Hit".into(),
                kind: Kind::AudioClip,
                asset: id,
            }],
            ..Default::default()
        };
        assert_eq!(table.find("Hit", Some(&Kind::AudioClip)), Some(id));
        assert_eq!(table.find("hit", None), None);
        assert_eq!(table.find("Hit", Some(&Kind::Texture)), None);
        assert_eq!(DataTable::parse(&table.encode().unwrap()).unwrap(), table);
        table.rows.push(table.rows[0].clone());
        assert!(table.validate().unwrap_err().contains("Duplicate"));
        table.rows.pop();
        table.rows[0].kind = Kind::Texture;
        assert!(table.validate().is_err());
        table.rows[0].key = " bad ".into();
        assert!(table.validate().is_err());
    }
    #[test]
    fn missing_conflicting_and_wrong_kind_assets_fail() {
        let id = Uuid::new_v4();
        let table = DataTable {
            rows: vec![Row {
                key: "Hit".into(),
                kind: Kind::AudioClip,
                asset: id,
            }],
            ..Default::default()
        };
        assert!(
            table
                .validate_assets(&Index::default())
                .unwrap_err()
                .contains("Hit")
        );
        let record = Record {
            path: "ignored".into(),
            revision: "rev".into(),
            meta: assets::Metadata {
                version: 2,
                id,
                kind: Kind::Texture,
                importer_version: 1,
                source: "assets/a".into(),
                source_hash: String::new(),
                settings: crate::import_settings::Settings::Texture,
                extra: Default::default(),
            },
        };
        let mut index = Index::default();
        index.assets.insert(id, vec![record.clone()]);
        assert!(
            table
                .validate_assets(&index)
                .unwrap_err()
                .contains("expected AudioClip")
        );
        index.assets.insert(id, vec![record.clone(), record]);
        assert!(
            table
                .validate_assets(&index)
                .unwrap_err()
                .contains("Conflicting")
        );
    }

    #[test]
    fn reachable_tables_cook_unattached_audio_and_reject_stale_saves() {
        let root = crate::workspace::tests::temp("data-table-resources");
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(
            root.join("assets/tone.wav"),
            crate::audio_import::test_wav(),
        )
        .unwrap();
        let clip = assets::commit(
            assets::prepare(
                &root,
                "assets/tone.wav",
                "assets/tone.epokasset",
                Default::default(),
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let table = DataTable {
            rows: vec![Row {
                key: "Hit".into(),
                kind: Kind::AudioClip,
                asset: clip,
            }],
            ..Default::default()
        };
        let id = create(&root, "assets/Sounds.epokasset", &table).unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let extra = vec![(
            crate::reflection_schema::Type::AssetRef {
                kind: "DataTable".into(),
            },
            serde_json::json!(id),
        )];
        let resources = crate::blueprint_refs::resources(
            &[],
            &[],
            &crate::actor_document::tests::registry(),
            &index,
            &extra,
        )
        .unwrap();
        assert_eq!(crate::audio::clip_ids(&resources), vec![clip]);
        assert_eq!(resources.reference_tables[&id], table);
        let merged = crate::scene_bank::resources(&[crate::scene::Scene::default(), resources]);
        assert_eq!(merged.reference_tables.len(), 1);
        let record = index.resolve(id).unwrap();
        let mut edited = table.clone();
        edited.rows[0].key = "Impact".into();
        save(record, &edited, &index).unwrap();
        assert!(
            save(record, &table, &index).is_err(),
            "an external edit must not be lost"
        );
        assert_eq!(document(record).unwrap().rows[0].key, "Impact");
    }
}
