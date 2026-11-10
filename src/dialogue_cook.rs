//! Localized, bounded conversation resources shared by target and host staging.
#[cfg(test)]
use crate::dialogue_text::Metrics;
use crate::{
    assets::{self, Index},
    dialogue_asset as asset, dialogue_compile as graph,
    dialogue_text::{self, Page},
    localization::StringTable,
};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

/// Exact label rectangles supplied by the shared presenter composition. Choices
/// and names must fit one page; line/prompt content can advance across pages.
#[derive(Clone, Copy, Debug)]
#[cfg(test)]
pub struct Areas {
    pub line: [u16; 2],
    pub prompt: [u16; 2],
    pub choice: [u16; 2],
    pub name: [u16; 2],
}
#[derive(Clone, Debug)]
pub struct Style {
    pub id: Uuid,
    pub source: asset::DialogueStyle,
}
#[derive(Clone, Debug)]
pub struct Variant {
    pub graph: graph::Compiled,
    pub style: Style,
    pub texts: Vec<Text>,
    pub widget: Option<crate::dialogue_layout::Profile>,
}
#[derive(Clone, Debug, Default)]
pub struct Bundle {
    pub variants: Vec<Variant>,
    pub languages: Vec<String>,
    pub dependencies: BTreeMap<String, String>,
}
#[derive(Default)]
pub struct Selection {
    pub assets: BTreeMap<Uuid, assets::Kind>,
    pub pairs: BTreeSet<(Uuid, Uuid)>,
}
impl Selection {
    pub fn prepare(
        files: &[crate::blueprint_asset::AssetFile],
        scenes: &[crate::scene::Scene],
        registry: &crate::blueprint::Registry,
        index: &Index,
        extra: &[(crate::reflection_schema::Type, serde_json::Value)],
    ) -> Result<Self, String> {
        let selected = crate::blueprint_refs::asset_refs(files, scenes, registry, index, extra)?;
        if !selected
            .values()
            .any(|kind| *kind == assets::Kind::Dialogue)
        {
            return Ok(Self {
                assets: selected,
                pairs: BTreeSet::new(),
            });
        }
        let model = registry.model().map_err(|e| {
            e.into_iter()
                .map(|e| e.message)
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        let mut pairs = BTreeSet::new();
        let mut add = |class: &str, values: &BTreeMap<String, serde_json::Value>| {
            let Some(class) = registry
                .classes
                .get(class)
                .or_else(|| registry.named(class))
            else {
                return;
            };
            if !model.is_a(&class.id, "aa343ddf-5098-5b32-b993-d8b9b619d455") {
                return;
            }
            let properties = registry.properties(&class.cpp_name);
            let value = |name: &str| {
                values.get(name).or_else(|| {
                    properties
                        .iter()
                        .find(|p| p.name == name)
                        .map(|p| &p.default)
                })
            };
            let parse = |value: Option<&serde_json::Value>| {
                value
                    .and_then(|v| v.as_str())
                    .and_then(|v| Uuid::parse_str(v).ok())
            };
            if let Some(conversation) = parse(value("conversation")) {
                pairs.insert((conversation, parse(value("style")).unwrap_or(Uuid::nil())));
            }
        };
        for class in registry.classes.values() {
            add(&class.cpp_name, &BTreeMap::new());
        }
        for scene in scenes {
            for actor in &scene.actors {
                for (class, values) in std::iter::once((&actor.class, &actor.properties))
                    .chain(actor.components.iter().map(|c| (&c.class, &c.properties)))
                {
                    add(class.class_id.as_deref().unwrap_or(&class.name), values);
                }
            }
        }
        let literals = files
            .iter()
            .flat_map(|f| crate::blueprint_refs::graph_resources(&f.asset))
            .chain(extra.iter().map(|(ty, value)| (ty, value)))
            .collect::<Vec<_>>();
        let dynamic_styles=literals.iter().any(|(ty,_)|matches!(ty,crate::reflection_schema::Type::AssetRef{kind} if kind=="DialogueStyle"));
        let dynamic_default = literals.iter().any(|(ty, value)| {
            matches!(ty,crate::reflection_schema::Type::AssetRef{kind} if kind=="DialogueStyle")
                && value.is_null()
        });
        let dynamic_conversations = literals
            .iter()
            .filter_map(|(ty, value)| {
                matches!(ty,crate::reflection_schema::Type::AssetRef{kind} if kind=="Dialogue")
                    .then(|| value.as_str().and_then(|s| Uuid::parse_str(s).ok()))
                    .flatten()
            })
            .collect::<BTreeSet<_>>();
        let styles = selected
            .iter()
            .filter(|(_, kind)| **kind == assets::Kind::DialogueStyle)
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        for (id, kind) in &selected {
            if *kind != assets::Kind::Dialogue {
                continue;
            }
            let configured = pairs.iter().any(|(conversation, _)| conversation == id);
            if !configured || dynamic_styles || dynamic_conversations.contains(id) {
                for style in &styles {
                    pairs.insert((*id, *style));
                }
                // Opaque/native consumers have no authored pair. Preserve the
                // default resource as well as every selected explicit style.
                if styles.is_empty() || dynamic_default || !configured {
                    pairs.insert((*id, Uuid::nil()));
                }
            }
        }
        Ok(Self {
            assets: selected,
            pairs,
        })
    }
}
impl Bundle {
    pub fn prepare_widgets(
        selection: &Selection,
        index: &Index,
        registry: &crate::blueprint::Registry,
        display: [u16; 2],
        fonts: &BTreeMap<Uuid, std::sync::Arc<crate::font_asset::FontData>>,
        scenes: &[crate::scene::Scene],
    ) -> Result<Self, String> {
        let profiles = crate::dialogue_layout::Profile::collect(scenes, registry)?;
        Self::prepare_pairs(
            &selection.assets,
            index,
            registry,
            display,
            fonts,
            Some(&selection.pairs),
            &profiles,
        )
    }
    #[cfg(test)]
    pub fn prepare_selected(
        selection: &Selection,
        index: &Index,
        registry: &crate::blueprint::Registry,
        display: [u16; 2],
        fonts: &BTreeMap<Uuid, std::sync::Arc<crate::font_asset::FontData>>,
    ) -> Result<Self, String> {
        Self::prepare_pairs(
            &selection.assets,
            index,
            registry,
            display,
            fonts,
            Some(&selection.pairs),
            &Default::default(),
        )
    }
    pub fn install_languages(&self, scene: &mut crate::scene::Scene) {
        if !self.variants.is_empty() && scene.strings.is_none() {
            scene.strings = Some(std::sync::Arc::new(StringTable {
                languages: self.languages.clone(),
                entries: vec![],
                ..Default::default()
            }));
        }
    }
    #[cfg(test)]
    pub fn prepare(
        selected: &BTreeMap<Uuid, assets::Kind>,
        index: &Index,
        registry: &crate::blueprint::Registry,
        display: [u16; 2],
        fonts: &BTreeMap<Uuid, std::sync::Arc<crate::font_asset::FontData>>,
    ) -> Result<Self, String> {
        Self::prepare_pairs(
            selected,
            index,
            registry,
            display,
            fonts,
            None,
            &Default::default(),
        )
    }
    fn prepare_pairs(
        selected: &BTreeMap<Uuid, assets::Kind>,
        index: &Index,
        registry: &crate::blueprint::Registry,
        _display: [u16; 2],
        fonts: &BTreeMap<Uuid, std::sync::Arc<crate::font_asset::FontData>>,
        pairs: Option<&BTreeSet<(Uuid, Uuid)>>,
        profiles: &crate::dialogue_layout::Collection,
    ) -> Result<Self, String> {
        let graphs = selected
            .iter()
            .filter(|(_, kind)| **kind == assets::Kind::Dialogue)
            .map(|(id, _)| {
                let source = asset::document(index.resolve(*id)?)?;
                graph::compile(*id, &source, registry, index)
            })
            .collect::<Result<Vec<_>, String>>()?;
        if graphs.is_empty() {
            return Ok(Self::default());
        }
        let locales = Locales::prepare(
            index,
            &graphs
                .iter()
                .flat_map(|g| g.texts.iter().map(|t| t.table))
                .collect(),
        )?;
        let mut styles = vec![];
        for (id, kind) in selected {
            if *kind == assets::Kind::DialogueStyle {
                styles.push((*id, asset::style_document(index.resolve(*id)?)?));
            }
        }
        if styles.is_empty() || pairs.is_some_and(|p| p.iter().any(|(_, style)| style.is_nil())) {
            styles.push((Uuid::nil(), asset::DialogueStyle::default()));
        }
        let mut result = Self {
            languages: locales.names.clone(),
            dependencies: locales.dependencies.clone(),
            ..Default::default()
        };
        // IDs and all resource revisions are inputs, including noninitial nodes.
        let mut compact = BTreeMap::new();
        for id in selected.keys() {
            let key = crate::blueprint_refs::compact_id(&id.to_string());
            if key == 0 || compact.insert(key, *id).is_some_and(|old| old != *id) {
                return Err(format!("Dialogue resource identity collision at {id}"));
            }
            result.dependencies.insert(
                format!("asset:{id}"),
                assets::cache_key(&index.resolve(*id)?.meta),
            );
        }
        for (id, source) in styles {
            if pairs.is_some_and(|p| !p.iter().any(|(_, style)| *style == id)) {
                continue;
            }
            source.validate_resources(index)?;
            for graph in &graphs {
                if pairs.is_some_and(|p| !p.contains(&(graph.asset, id))) {
                    continue;
                }
                if !profiles.profiles.is_empty() {
                    let before = result.variants.len();
                    let mut failure = None;
                    for profile in &profiles.profiles {
                        let texts = if profile.supports(graph) {
                            profile.texts(graph, &locales, fonts)
                        } else {
                            Err(
                                "Dialogue choice nodes require four bound choice Text controls"
                                    .into(),
                            )
                        };
                        let texts = match texts {
                            Ok(texts) => texts,
                            Err(error) => {
                                if profiles.bound.contains(&(graph.asset, id, profile.key())) {
                                    return Err(error);
                                }
                                failure = Some(error);
                                continue;
                            }
                        };
                        result.variants.push(Variant {
                            texts,
                            graph: graph.clone(),
                            style: Style {
                                id,
                                source: source.clone(),
                            },
                            widget: Some(profile.clone()),
                        });
                    }
                    if result.variants.len() == before {
                        return Err(failure.unwrap_or_else(|| {
                            "No dialogue widget can display this conversation".into()
                        }));
                    }
                    continue;
                }
                result.variants.push(Variant {
                    texts: unpaged_texts(graph, &locales)?,
                    graph: graph.clone(),
                    style: Style {
                        id,
                        source: source.clone(),
                    },
                    widget: None,
                });
            }
        }
        Ok(result)
    }
    pub fn header(&self, resources: &crate::scene::Scene) -> Result<String, String> {
        let textures = crate::texture::ids(resources)
            .into_iter()
            .enumerate()
            .map(|(i, id)| {
                i16::try_from(i)
                    .map(|i| (id, i))
                    .map_err(|_| "Dialogue texture index exceeds int16")
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let audio = crate::audio::clip_ids(resources);
        let mut out = String::from("#if defined(EPOK_DIALOGUE)\n#include \"dialogue.hpp\"\n");
        let mut rows = vec![];
        for variant in &self.variants {
            let widget = variant.widget.as_ref().map_or(0, |profile| profile.key());
            out += &header_for_widget(
                &variant.graph,
                &variant.texts,
                variant.style.id,
                &textures,
                widget,
            )?;
            let ns = namespace(&variant.graph, variant.style.id, widget);
            let style = &variant.style.source;
            let sound = |id: Option<Uuid>| -> Result<i32, String> {
                id.map(|id| {
                    audio
                        .iter()
                        .position(|v| *v == id)
                        .ok_or_else(|| format!("Unstaged dialogue sound {id}"))
                })
                .transpose()
                .map(|i| i.map_or(-1, |i| i as i32))
            };
            out += &format!(
                "namespace epok::dialogue::{ns} {{inline constexpr Style style={{{{{},{}}},{},{},{}}};}}\n",
                style.line_policy.milliseconds_per_glyph,
                style.line_policy.auto_advance_ms.unwrap_or(0),
                sound(style.open_sound)?,
                sound(style.advance_sound)?,
                sound(style.close_sound)?,
            );
            let style_id = if variant.style.id.is_nil() {
                0
            } else {
                crate::blueprint_refs::compact_id(&variant.style.id.to_string())
            };
            rows.push(format!(
                "{{UINT64_C({}),UINT64_C({style_id}),&{ns}::conversation,&{ns}::style,UINT64_C({widget})}}",
                crate::blueprint_refs::compact_id(&variant.graph.asset.to_string())
            ));
        }
        let count = rows.len();
        if rows.is_empty() {
            rows.push("{}".into());
        } else {
            out.insert_str(0, "#define EPOK_DIALOGUE_USED 1\n");
        }
        out += &format!(
            "namespace epok::dialogue {{inline const Resource resources[]={{{}}};[[gnu::used]] inline const size_t resource_count={count};}}\n#endif\n",
            rows.join(",")
        );
        Ok(out)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Text {
    pub pages: Vec<Page>,
    pub offsets: Vec<u16>,
}
#[derive(Clone, Debug)]
pub struct Locales {
    pub names: Vec<String>,
    tables: BTreeMap<Uuid, (StringTable, Vec<usize>)>,
    pub dependencies: BTreeMap<String, String>,
}
fn locale_key(name: &str) -> Result<String, String> {
    let value = name.trim().to_lowercase();
    if value.is_empty() {
        return Err("Dialogue languages need nonempty names".into());
    }
    Ok(value)
}
fn locale_indices(names: &[String]) -> Result<BTreeMap<String, usize>, String> {
    let mut indices = BTreeMap::new();
    for (index, name) in names.iter().enumerate() {
        if indices.insert(locale_key(name)?, index).is_some() {
            return Err(format!(
                "Dialogue language '{name}' is ambiguous or duplicated"
            ));
        }
    }
    Ok(indices)
}
impl Locales {
    /// The project HUD table defines language indices when present; otherwise
    /// the project's lowest table UUID does. This stays fixed when Play selects
    /// fewer scenes. Match referenced columns by locale name.
    pub fn prepare(index: &Index, tables: &BTreeSet<Uuid>) -> Result<Self, String> {
        let mut loaded = BTreeMap::new();
        let mut dependencies = BTreeMap::new();
        for id in tables {
            let record = index.resolve(*id)?;
            loaded.insert(*id, crate::localization::document(record)?);
            dependencies.insert(format!("asset:{id}"), assets::cache_key(&record.meta));
        }
        let names =
            if let Some((record, table)) = crate::localization::project_table_checked(index)? {
                dependencies.insert(
                    format!("asset:{}", record.meta.id),
                    assets::cache_key(&record.meta),
                );
                table.languages
            } else if let Some(record) = index
                .usable()
                .filter(|r| r.meta.kind == assets::Kind::StringTable)
                .min_by_key(|r| r.meta.id)
            {
                dependencies.insert(
                    format!("asset:{}", record.meta.id),
                    assets::cache_key(&record.meta),
                );
                crate::localization::document(record)?.languages
            } else {
                vec!["English".into()]
            };
        let canonical = locale_indices(&names)?;
        let mut result = BTreeMap::new();
        for (id, table) in loaded {
            let actual = locale_indices(&table.languages)?;
            if actual.keys().ne(canonical.keys()) {
                let missing = canonical
                    .keys()
                    .filter(|key| !actual.contains_key(*key))
                    .cloned()
                    .collect::<Vec<_>>();
                let extra = actual
                    .keys()
                    .filter(|key| !canonical.contains_key(*key))
                    .cloned()
                    .collect::<Vec<_>>();
                return Err(format!(
                    "Dialogue String Table {id}: languages differ from the project (missing {missing:?}, extra {extra:?})"
                ));
            }
            let columns = names
                .iter()
                .map(|name| locale_key(name).map(|key| actual[&key]))
                .collect::<Result<Vec<_>, _>>()?;
            result.insert(id, (table, columns));
        }
        Ok(Self {
            names,
            tables: result,
            dependencies,
        })
    }
    pub fn text(&self, key: &asset::TextKey, language: usize) -> Result<&str, String> {
        let (table, columns) = self
            .tables
            .get(&key.table)
            .ok_or_else(|| format!("Unselected dialogue String Table {}", key.table))?;
        let column = *columns
            .get(language)
            .ok_or("Invalid dialogue language index")?;
        // Preserve StringTable's authored first-column fallback for empty cells.
        table.lookup(&key.key, column).ok_or_else(|| {
            format!(
                "Dialogue String Table {} has no key '{}'",
                key.table, key.key
            )
        })
    }
}

/// Low-level Session users can consume a short payload without a widget.
/// Longer text requires an authored profile; no implicit visual area is invented.
fn unpaged_texts(graph: &graph::Compiled, locales: &Locales) -> Result<Vec<Text>, String> {
    graph
        .texts
        .iter()
        .map(|key| {
            let mut result = Text {
                pages: vec![],
                offsets: vec![0],
            };
            for (language, name) in locales.names.iter().enumerate() {
                result.pages.push(
                    dialogue_text::unpaged(locales.text(key, language)?).map_err(|e| {
                        format!("Dialogue {} / {} / {name}: {e}", key.table, key.key)
                    })?,
                );
                result.offsets.push(
                    u16::try_from(result.pages.len()).map_err(|_| "Too many dialogue languages")?,
                );
            }
            Ok(result)
        })
        .collect()
}
#[cfg(test)]
pub fn texts(
    compiled: &graph::Compiled,
    locales: &Locales,
    metrics: &Metrics,
    areas: Areas,
) -> Result<Vec<Text>, String> {
    let mut bounds = vec![([u16::MAX; 2], false); compiled.texts.len()];
    let mut constrain = |index: u16, size: [u16; 2], single: bool| {
        let item = &mut bounds[usize::from(index)];
        item.0 = [item.0[0].min(size[0]), item.0[1].min(size[1])];
        item.1 |= single;
    };
    for role in &compiled.roles {
        if let Some(name) = role.display_name {
            constrain(name, areas.name, true);
        }
    }
    for node in &compiled.nodes {
        match &node.kind {
            graph::NodeKind::Line { text, .. } => constrain(*text, areas.line, false),
            graph::NodeKind::Choice {
                prompt, options, ..
            } => {
                if let Some(prompt) = prompt {
                    constrain(*prompt, areas.prompt, false);
                }
                for option in options {
                    constrain(option.text, areas.choice, true);
                }
            }
            _ => {}
        }
    }
    compiled
        .texts
        .iter()
        .zip(bounds)
        .map(|(key, (size, single))| {
            let mut result = Text { pages: vec![], offsets: vec![0] };
            for (language, name) in locales.names.iter().enumerate() {
                let context = |error| format!("Dialogue {} / {} / {name}: {error}", key.table, key.key);
                let pages = dialogue_text::paginate(
                    locales.text(key, language)?, metrics, size[0], size[1],
                ).map_err(context)?;
                if single && pages.len() != 1 {
                    return Err(context("choice or participant name does not fit its label; shorten the translation or enlarge the presenter area".into()));
                }
                result.pages.extend(pages);
                result.offsets.push(u16::try_from(result.pages.len()).map_err(|_| context("combined translations exceed 65535 pages".into()))?);
            }
            Ok(result)
        })
        .collect()
}

fn literal(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .bytes()
            .map(|b| format!("\\{b:03o}"))
            .collect::<String>()
    )
}
/// Emit immutable native tables and typed reflected method calls. Every pointer
/// refers to this namespace's own storage; no runtime text/graph parser exists.
#[cfg(test)]
pub fn header(
    compiled: &graph::Compiled,
    texts: &[Text],
    style: Uuid,
    texture_indices: &BTreeMap<Uuid, i16>,
) -> Result<String, String> {
    header_for_widget(compiled, texts, style, texture_indices, 0)
}
fn namespace(compiled: &graph::Compiled, style: Uuid, widget: u64) -> String {
    let base = format!("asset_{}_{}", compiled.asset.simple(), style.simple());
    if widget == 0 {
        base
    } else {
        format!("{base}_widget_{widget:016x}")
    }
}
fn header_for_widget(
    compiled: &graph::Compiled,
    texts: &[Text],
    style: Uuid,
    texture_indices: &BTreeMap<Uuid, i16>,
    widget: u64,
) -> Result<String, String> {
    if texts.len() != compiled.texts.len() {
        return Err("Dialogue text table does not match its compiled graph".into());
    }
    let name = namespace(compiled, style, widget);
    let mut out = format!("namespace epok::dialogue::{name} {{\n");
    for (index, text) in texts.iter().enumerate() {
        if text.offsets.len() < 2
            || text.pages.is_empty()
            || text.pages.len() > usize::from(u16::MAX)
            || text.offsets[0] != 0
            || text.offsets.last().copied() != Some(text.pages.len() as u16)
            || text.offsets.windows(2).any(|p| p[0] >= p[1])
            || text
                .pages
                .iter()
                .any(|p| p.text.len() > dialogue_text::MAX_PAGE_BYTES || p.text.contains('\0'))
        {
            return Err("Invalid bounded dialogue page table".into());
        }
        out += &format!(
            "inline constexpr Page pages_{index}[]={{{}}};\n",
            text.pages
                .iter()
                .map(|p| format!("{{{},{}}}", literal(&p.text), p.glyphs))
                .collect::<Vec<_>>()
                .join(",")
        );
        out += &format!(
            "inline constexpr uint16_t languages_{index}[]={{{}}};\n",
            text.offsets
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
    }
    let rows = texts
        .iter()
        .enumerate()
        .map(|(i, t)| {
            format!(
                "{{pages_{i},languages_{i},{},{}}}",
                t.pages.len(),
                t.offsets.len() - 1
            )
        })
        .collect::<Vec<_>>();
    out += &format!(
        "inline constexpr Text texts[]={{{}}};\n",
        if rows.is_empty() {
            "{}".into()
        } else {
            rows.join(",")
        }
    );
    let roles = compiled
        .roles
        .iter()
        .map(|r| {
            let portrait = r
                .portrait
                .map(|id| {
                    texture_indices
                        .get(&id)
                        .copied()
                        .ok_or_else(|| format!("Unstaged dialogue portrait {id}"))
                })
                .transpose()?
                .unwrap_or(-1);
            Ok(format!(
                "{{{}u,{}, {portrait}}}",
                r.id,
                r.display_name.unwrap_or(u16::MAX)
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    out += &format!(
        "inline constexpr Role roles[]={{{}}};\n",
        if roles.is_empty() {
            "{}".into()
        } else {
            roles.join(",")
        }
    );
    let mut bindings = vec![];
    for (i, binding) in compiled.bindings.iter().enumerate() {
        let class = &binding.cpp_class;
        let method = &binding.cpp_method;
        let class_id = crate::blueprint_refs::compact_id(&binding.class_id);
        let entry = match binding.kind {
            asset::BindingKind::Condition => {
                out += &format!(
                    "inline bool condition_{i}(ObjectId receiver,ObjectId initiator){{const auto* object=active_object_registry?active_object_registry->resolve<{class}>(receiver):nullptr;return object&&object->{method}(initiator);}}\n"
                );
                format!(
                    "{{{},UINT64_C({class_id}),condition_{i},nullptr}}",
                    binding.role
                )
            }
            asset::BindingKind::Action => {
                out += &format!(
                    "inline void action_{i}(ObjectId receiver,ObjectId initiator,uint32_t port){{auto* object=active_object_registry?active_object_registry->resolve<{class}>(receiver):nullptr;if(object)object->{method}(initiator,port);}}\n"
                );
                format!(
                    "{{{},UINT64_C({class_id}),nullptr,action_{i}}}",
                    binding.role
                )
            }
        };
        bindings.push(entry);
    }
    out += &format!(
        "inline constexpr Binding bindings[]={{{}}};\n",
        if bindings.is_empty() {
            "{}".into()
        } else {
            bindings.join(",")
        }
    );
    let mut nodes = vec![];
    for (i, node) in compiled.nodes.iter().enumerate() {
        let (kind, speaker, text, next, otherwise, binding, options, count, reveal, auto) =
            match &node.kind {
                graph::NodeKind::Line {
                    speaker,
                    text,
                    next,
                    policy,
                } => (
                    "Line",
                    speaker.unwrap_or(u8::MAX),
                    *text,
                    *next,
                    u16::MAX,
                    u16::MAX,
                    "nullptr".into(),
                    0,
                    policy
                        .as_ref()
                        .map_or(u16::MAX, |p| p.milliseconds_per_glyph),
                    policy
                        .as_ref()
                        .map_or(u32::MAX, |p| p.auto_advance_ms.unwrap_or(0)),
                ),
                graph::NodeKind::Choice {
                    speaker,
                    prompt,
                    options,
                } => {
                    out += &format!(
                        "inline constexpr Option options_{i}[]={{{}}};\n",
                        options
                            .iter()
                            .map(|o| format!(
                                "{{{}u,{},{},{}}}",
                                o.id,
                                o.text,
                                o.next,
                                o.condition.map_or(u16::MAX, u16::from)
                            ))
                            .collect::<Vec<_>>()
                            .join(",")
                    );
                    (
                        "Choice",
                        speaker.unwrap_or(u8::MAX),
                        prompt.unwrap_or(u16::MAX),
                        u16::MAX,
                        u16::MAX,
                        u16::MAX,
                        format!("options_{i}"),
                        options.len(),
                        u16::MAX,
                        u32::MAX,
                    )
                }
                graph::NodeKind::Branch { condition, yes, no } => (
                    "Branch",
                    u8::MAX,
                    u16::MAX,
                    *yes,
                    *no,
                    u16::from(*condition),
                    "nullptr".into(),
                    0,
                    u16::MAX,
                    u32::MAX,
                ),
                graph::NodeKind::Event { action, next } => (
                    "Event",
                    u8::MAX,
                    u16::MAX,
                    *next,
                    u16::MAX,
                    u16::from(*action),
                    "nullptr".into(),
                    0,
                    u16::MAX,
                    u32::MAX,
                ),
                graph::NodeKind::End => (
                    "End",
                    u8::MAX,
                    u16::MAX,
                    u16::MAX,
                    u16::MAX,
                    u16::MAX,
                    "nullptr".into(),
                    0,
                    u16::MAX,
                    u32::MAX,
                ),
            };
        nodes.push(format!("{{{}u,Kind::{kind},{speaker},{text},{next},{otherwise},{binding},{options},{count},{reveal},{auto}u}}",node.id));
    }
    out += &format!("inline constexpr Node nodes[]={{{}}};\n", nodes.join(","));
    out += &format!(
        "inline constexpr Conversation conversation={{UINT64_C({}),roles,{},bindings,{},texts,{},nodes,{},{}}};\n}}\n",
        crate::blueprint_refs::compact_id(&compiled.asset.to_string()),
        compiled.roles.len(),
        compiled.bindings.len(),
        texts.len(),
        compiled.nodes.len(),
        compiled.entry
    );
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::localization::{self, Entry};
    fn project(name: &str) -> std::path::PathBuf {
        let root = crate::workspace::tests::temp(name);
        crate::workspace::create(&root, name, crate::workspace::Template::Basic).unwrap();
        root
    }
    fn table(root: &std::path::Path, path: &str, names: &[&str], values: &[&str]) -> Uuid {
        localization::create(
            root,
            path,
            &StringTable {
                pageable: true,
                languages: names.iter().map(|s| (*s).into()).collect(),
                entries: vec![Entry {
                    key: "text".into(),
                    values: values.iter().map(|s| (*s).into()).collect(),
                }],
                ..Default::default()
            },
        )
        .unwrap()
    }
    #[test]
    fn dialogue_locales_match_names_and_preserve_authored_fallback() {
        let root = project("dialogue-locale-order");
        localization::create(
            &root,
            "assets/hud.epokasset",
            &StringTable {
                languages: vec!["English".into(), "Español".into()],
                ..Default::default()
            },
        )
        .unwrap();
        let a = table(
            &root,
            "assets/a.epokasset",
            &["Español", "English"],
            &["Hola", "Hello"],
        );
        let b = table(
            &root,
            "assets/b.epokasset",
            &[" english ", "español"],
            &["World", ""],
        );
        let index = assets::scan(&root, &mut Default::default());
        let ids = BTreeSet::from([a, b]);
        let locales = Locales::prepare(&index, &ids).unwrap();
        assert_eq!(locales.names, ["English", "Español"]);
        let key = |table| asset::TextKey {
            table,
            key: "text".into(),
        };
        assert_eq!(locales.text(&key(a), 0).unwrap(), "Hello");
        assert_eq!(locales.text(&key(a), 1).unwrap(), "Hola");
        assert_eq!(locales.text(&key(b), 1).unwrap(), "World");
        assert_eq!(locales.dependencies.len(), 3);
        let mut changed = localization::document(index.resolve(a).unwrap()).unwrap();
        changed.languages.reverse();
        changed.entries[0].values.reverse();
        localization::save(index.resolve(a).unwrap(), &changed).unwrap();
        let fresh = assets::scan(&root, &mut Default::default());
        let reordered = Locales::prepare(&fresh, &ids).unwrap();
        assert_eq!(reordered.text(&key(a), 0).unwrap(), "Hello");
        assert_eq!(reordered.text(&key(a), 1).unwrap(), "Hola");
        assert_ne!(locales.dependencies, reordered.dependencies);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn dialogue_language_indices_stay_fixed_when_play_selects_fewer_scenes() {
        let root = project("dialogue-language-scene-selection");
        let a = table(
            &root,
            "assets/a.epokasset",
            &["English", "Español"],
            &["Hello", "Hola"],
        );
        let b = table(
            &root,
            "assets/b.epokasset",
            &["Español", "English"],
            &["Hola", "Hello"],
        );
        let index = assets::scan(&root, &mut Default::default());
        let whole = Locales::prepare(&index, &BTreeSet::from([a, b])).unwrap();
        let secondary = a.max(b);
        let selected = Locales::prepare(&index, &BTreeSet::from([secondary])).unwrap();
        assert_eq!(whole.names, selected.names);
        assert!(
            selected
                .dependencies
                .contains_key(&format!("asset:{}", a.min(b)))
        );
        let key = asset::TextKey {
            table: secondary,
            key: "text".into(),
        };
        for i in 0..2 {
            assert_eq!(
                whole.text(&key, i).unwrap(),
                selected.text(&key, i).unwrap()
            );
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn dialogue_locales_diagnose_missing_extra_and_ambiguous_columns() {
        let root = project("dialogue-locale-errors");
        localization::create(
            &root,
            "assets/hud.epokasset",
            &StringTable {
                languages: vec!["English".into(), "Español".into()],
                ..Default::default()
            },
        )
        .unwrap();
        let a = table(
            &root,
            "assets/a.epokasset",
            &["English", "French"],
            &["Hello", "Bonjour"],
        );
        let index = assets::scan(&root, &mut Default::default());
        let error = Locales::prepare(&index, &BTreeSet::from([a])).unwrap_err();
        assert!(
            error.contains("missing") && error.contains("español") && error.contains("french"),
            "{error}"
        );
        let mut changed = localization::document(index.resolve(a).unwrap()).unwrap();
        changed.languages = vec!["English".into(), " english ".into()];
        localization::save(index.resolve(a).unwrap(), &changed).unwrap();
        let fresh = assets::scan(&root, &mut Default::default());
        assert!(
            Locales::prepare(&fresh, &BTreeSet::from([a]))
                .unwrap_err()
                .contains("ambiguous")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn dialogue_cook_pages_lines_but_rejects_truncated_choice_labels() {
        let root = project("dialogue-cooked-pages");
        let a = table(
            &root,
            "assets/a.epokasset",
            &["English", "Español"],
            &[&"Á".repeat(600), &"BC".repeat(400)],
        );
        let key = asset::TextKey {
            table: a,
            key: "text".into(),
        };
        let index = assets::scan(&root, &mut Default::default());
        let locales = Locales::prepare(&index, &BTreeSet::from([a])).unwrap();
        let mut compiled = graph::Compiled {
            asset: Uuid::new_v4(),
            entry: 0,
            roles: vec![],
            bindings: vec![],
            texts: vec![key],
            nodes: vec![
                graph::Node {
                    id: 1,
                    source: Uuid::new_v4(),
                    kind: graph::NodeKind::Line {
                        speaker: None,
                        text: 0,
                        next: 1,
                        policy: None,
                    },
                },
                graph::Node {
                    id: 2,
                    source: Uuid::new_v4(),
                    kind: graph::NodeKind::End,
                },
            ],
        };
        let areas = Areas {
            line: [160, 48],
            prompt: [160, 48],
            choice: [144, 16],
            name: [80, 16],
        };
        let pages = texts(&compiled, &locales, &Metrics::builtin(), areas).unwrap();
        assert_eq!(pages[0].offsets, [0, 10, 24]);
        assert_eq!(
            pages[0]
                .pages
                .iter()
                .map(|p| u32::from(p.glyphs))
                .sum::<u32>(),
            1400
        );
        let style = Uuid::new_v4();
        let emitted = header(&compiled, &pages, style, &Default::default()).unwrap();
        assert!(
            emitted.contains("inline constexpr Conversation")
                && emitted.contains("65535,4294967295u")
        );
        assert_eq!(
            header(&compiled, &pages, style, &Default::default()).unwrap(),
            emitted
        );
        compiled.nodes[0].kind = graph::NodeKind::Choice {
            speaker: None,
            prompt: None,
            options: vec![graph::OptionRow {
                id: 3,
                text: 0,
                next: 1,
                condition: None,
            }],
        };
        let error = texts(&compiled, &locales, &Metrics::builtin(), areas).unwrap_err();
        assert!(
            error.contains("English") && error.contains("text") && error.contains("does not fit"),
            "{error}"
        );
        compiled.roles.push(graph::Role {
            id: 4,
            source: Uuid::new_v4(),
            display_name: None,
            portrait: Some(a),
        });
        assert!(
            header(&compiled, &pages, style, &Default::default())
                .unwrap_err()
                .contains("Unstaged dialogue portrait")
        );
        let mut bad = pages.clone();
        bad[0].pages[0].text = "x".repeat(512);
        assert!(
            header(&compiled, &bad, style, &BTreeMap::from([(a, 0)]))
                .unwrap_err()
                .contains("bounded")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn dialogue_resource_closure_stages_later_text_font_portrait_panel_and_audio() {
        let root = project("dialogue-resource-closure");
        std::fs::write(root.join("assets/panel.png"), crate::texture::tests::png()).unwrap();
        let panel = assets::commit(
            crate::texture::prepare(
                &root,
                "assets/panel.png",
                "assets/panel.epokasset",
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            root.join("assets/font.ttf"),
            include_bytes!("../resources/editor/Roboto-Regular.ttf"),
        )
        .unwrap();
        let font = assets::commit(
            crate::font_asset::prepare(
                &root,
                "assets/font.ttf",
                "assets/font.epokasset",
                Default::default(),
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            root.join("assets/sound.wav"),
            crate::audio_import::test_wav(),
        )
        .unwrap();
        let sound = assets::commit(
            assets::prepare(
                &root,
                "assets/sound.wav",
                "assets/sound.epokasset",
                Default::default(),
                None,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        let first = table(
            &root,
            "assets/first.epokasset",
            &["English", "Español"],
            &["First", "Primero"],
        );
        let later = table(
            &root,
            "assets/later.epokasset",
            &["Español", "English"],
            &["Después", "Later"],
        );
        // A nonpageable project table pins global language order independently
        // of either resource UUID or the order of the second table's columns.
        localization::create(
            &root,
            "assets/hud.epokasset",
            &StringTable {
                languages: vec!["English".into(), "Español".into()],
                ..Default::default()
            },
        )
        .unwrap();
        let role = Uuid::new_v4();
        let line = Uuid::new_v4();
        let second = Uuid::new_v4();
        let end = Uuid::new_v4();
        let key = |table| asset::TextKey {
            table,
            key: "text".into(),
        };
        let source = asset::DialogueAsset {
            version: asset::VERSION,
            entry: line,
            bindings: vec![],
            participants: vec![asset::Participant {
                id: role,
                label: "Speaker".into(),
                display_name: Some(key(later)),
                portrait: Some(panel),
            }],
            nodes: vec![
                asset::Node {
                    id: line,
                    kind: asset::NodeKind::Line {
                        speaker: Some(role),
                        text: key(first),
                        next: second,
                        policy: None,
                    },
                },
                asset::Node {
                    id: second,
                    kind: asset::NodeKind::Line {
                        speaker: Some(role),
                        text: key(later),
                        next: end,
                        policy: None,
                    },
                },
                asset::Node {
                    id: end,
                    kind: asset::NodeKind::End,
                },
            ],
        };
        let conversation = asset::create(&root, "assets/conversation.epokasset", &source).unwrap();
        let style = asset::create_style(
            &root,
            "assets/style.epokasset",
            &asset::DialogueStyle {
                advance_sound: Some(sound),
                ..Default::default()
            },
        )
        .unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let registry = crate::blueprint::registry_from_catalog(&root, &[]);
        use crate::reflection_schema::Type;
        let mut extra = vec![
            (
                Type::AssetRef {
                    kind: "Dialogue".into(),
                },
                serde_json::json!(conversation),
            ),
            (
                Type::AssetRef {
                    kind: "DialogueStyle".into(),
                },
                serde_json::json!(style),
            ),
        ];
        extra.push((
            Type::AssetRef {
                kind: "Font".into(),
            },
            serde_json::json!(font),
        ));
        let selected =
            crate::blueprint_refs::asset_refs(&[], &[], &registry, &index, &extra).unwrap();
        for id in [conversation, style, first, later, font, panel, sound] {
            assert!(selected.contains_key(&id));
        }
        let resources =
            crate::blueprint_refs::resources(&[], &[], &registry, &index, &extra).unwrap();
        assert_eq!(
            resources.actors.len(),
            3,
            "resource projections never create an actor per dialogue node or speaker"
        );
        assert!(resources.fonts.contains_key(&font) && resources.textures.contains_key(&panel));
        assert_eq!(crate::audio::clip_ids(&resources), vec![sound]);
        let bundle =
            Bundle::prepare(&selected, &index, &registry, [320, 240], &resources.fonts).unwrap();
        assert_eq!(bundle.variants.len(), 1);
        assert_eq!(bundle.languages, ["English", "Español"]);
        assert!(bundle.dependencies.contains_key(&format!("asset:{later}")));
        let emitted = bundle.header(&resources).unwrap();
        assert!(emitted.contains("resource_count=1") && emitted.contains("EPOK_DIALOGUE_USED"));
        let mut scene = crate::scene::Scene::default();
        bundle.install_languages(&mut scene);
        assert_eq!(scene.strings.as_ref().unwrap().languages, bundle.languages);
        assert!(scene.strings.as_ref().unwrap().entries.is_empty());
        let mut changed = localization::document(index.resolve(later).unwrap()).unwrap();
        changed.entries[0].values[1] = "Changed".into();
        localization::save(index.resolve(later).unwrap(), &changed).unwrap();
        let fresh = assets::scan(&root, &mut Default::default());
        let updated =
            Bundle::prepare(&selected, &fresh, &registry, [320, 240], &resources.fonts).unwrap();
        assert_ne!(bundle.dependencies, updated.dependencies);
        assert_ne!(emitted, updated.header(&resources).unwrap());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn dialogue_helpers_cook_each_instances_style_including_the_builtin_default() {
        let root = project("dialogue-style-pairs");
        let table_a = table(
            &root,
            "assets/a.epokasset",
            &["English"],
            &[&"A".repeat(28)],
        );
        let table_b = table(&root, "assets/b.epokasset", &["English"], &["Hi"]);
        let make = |table, choice| {
            let first = Uuid::new_v4();
            let end = Uuid::new_v4();
            asset::DialogueAsset {
                version: asset::VERSION,
                entry: first,
                participants: vec![],
                bindings: vec![],
                nodes: vec![
                    asset::Node {
                        id: first,
                        kind: if choice {
                            asset::NodeKind::Choice {
                                speaker: None,
                                prompt: None,
                                options: vec![asset::ChoiceOption {
                                    id: Uuid::new_v4(),
                                    text: asset::TextKey {
                                        table,
                                        key: "text".into(),
                                    },
                                    next: end,
                                    condition: None,
                                }],
                            }
                        } else {
                            asset::NodeKind::Line {
                                speaker: None,
                                text: asset::TextKey {
                                    table,
                                    key: "text".into(),
                                },
                                next: end,
                                policy: None,
                            }
                        },
                    },
                    asset::Node {
                        id: end,
                        kind: asset::NodeKind::End,
                    },
                ],
            }
        };
        let a = asset::create(
            &root,
            "assets/conversation-a.epokasset",
            &make(table_a, true),
        )
        .unwrap();
        let b = asset::create(
            &root,
            "assets/conversation-b.epokasset",
            &make(table_b, false),
        )
        .unwrap();
        let style = asset::create_style(
            &root,
            "assets/portrait-style.epokasset",
            &asset::DialogueStyle {
                line_policy: asset::LinePolicy {
                    milliseconds_per_glyph: 0,
                    auto_advance_ms: None,
                },
                ..Default::default()
            },
        )
        .unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let mut scene = crate::scene::Scene::default();
        scene.actors.clear();
        for (conversation, selected_style) in [(a, None), (b, Some(style))] {
            let mut actor = crate::scene::Actor::cube("Speaker".into());
            let mut helper = crate::actor_document::ComponentInstance::new(
                Uuid::new_v4(),
                crate::actor_document::ClassReference::new(
                    "epok::DialogueInteractionComponent",
                    "aa343ddf-5098-5b32-b993-d8b9b619d455",
                ),
                "Dialogue",
            );
            helper
                .properties
                .insert("conversation".into(), serde_json::json!(conversation));
            helper
                .properties
                .insert("style".into(), serde_json::json!(selected_style));
            actor.components.push(helper);
            scene.actors.push(actor);
        }
        let index = assets::scan(&root, &mut Default::default());
        let selected =
            Selection::prepare(&[], std::slice::from_ref(&scene), &registry, &index, &[]).unwrap();
        assert_eq!(
            selected.pairs,
            BTreeSet::from([(a, Uuid::nil()), (b, style)])
        );
        let bundle = Bundle::prepare_selected(
            &selected,
            &index,
            &registry,
            [320, 240],
            &Default::default(),
        )
        .unwrap();
        assert_eq!(bundle.variants.len(), 2);
        let opaque = Selection::prepare(
            &[],
            &[],
            &registry,
            &index,
            &[
                (
                    crate::reflection_schema::Type::AssetRef {
                        kind: "Dialogue".into(),
                    },
                    serde_json::json!(b),
                ),
                (
                    crate::reflection_schema::Type::AssetRef {
                        kind: "DialogueStyle".into(),
                    },
                    serde_json::json!(style),
                ),
            ],
        )
        .unwrap();
        assert_eq!(opaque.pairs, BTreeSet::from([(b, Uuid::nil()), (b, style)]));
        assert!(
            bundle
                .variants
                .iter()
                .any(|v| v.graph.asset == a && v.style.id.is_nil())
        );
        assert!(
            bundle
                .variants
                .iter()
                .any(|v| v.graph.asset == b && v.style.id == style)
        );
        // Timing assets carry no layout. A low-level session can cook both
        // combinations without accidentally imposing an unrelated widget's font.
        assert_eq!(
            Bundle::prepare(
                &selected.assets,
                &index,
                &registry,
                [320, 240],
                &Default::default()
            )
            .unwrap()
            .variants
            .len(),
            2
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn dialogue_widget_profiles_cook_named_localized_pages_at_both_resolutions() {
        let root = project("dialogue-style-presets");
        let strings = crate::localization::create(
            &root,
            "assets/strings.epokasset",
            &crate::localization::StringTable {
                pageable: true,
                entries: vec![
                    crate::localization::Entry {
                        key: "line".into(),
                        values: vec![format!(
                            "[color=#ffaa88]{}[/color]",
                            "Á long dialogue ".repeat(120)
                        )],
                    },
                    crate::localization::Entry {
                        key: "name".into(),
                        values: vec!["Keeper".into()],
                    },
                    crate::localization::Entry {
                        key: "choice".into(),
                        values: vec!["Continue".into()],
                    },
                ],
                ..Default::default()
            },
        )
        .unwrap();
        let mut source = asset::DialogueAsset::default();
        let entry = source.entry;
        let choice = Uuid::new_v4();
        let end = Uuid::new_v4();
        let role = Uuid::new_v4();
        let text = |key: &str| asset::TextKey {
            table: strings,
            key: key.into(),
        };
        source.participants.push(asset::Participant {
            id: role,
            label: "Speaker".into(),
            display_name: Some(text("name")),
            portrait: None,
        });
        source.nodes[0].kind = asset::NodeKind::Line {
            speaker: Some(role),
            text: text("line"),
            next: choice,
            policy: None,
        };
        source.nodes.push(asset::Node {
            id: choice,
            kind: asset::NodeKind::Choice {
                speaker: Some(role),
                prompt: None,
                options: (0..6)
                    .map(|_| asset::ChoiceOption {
                        id: Uuid::new_v4(),
                        text: text("choice"),
                        next: end,
                        condition: None,
                    })
                    .collect(),
            },
        });
        source.nodes.push(asset::Node {
            id: end,
            kind: asset::NodeKind::End,
        });
        assert_eq!(source.entry, entry);
        let conversation = asset::create(&root, "assets/conversation.epokasset", &source).unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        for display in [[320, 240], [640, 480]] {
            for presentation in asset::Presentation::ALL {
                let model = registry.model().unwrap();
                let mut scene = crate::scene::Scene {
                    display_size: display,
                    ..Default::default()
                };
                scene.actors.clear();
                scene.actors.push(
                    crate::class_composition::actor(
                        &model,
                        crate::dialogue_layout::PRESENTER,
                        "Widget",
                    )
                    .unwrap(),
                );
                crate::game_helpers::compose_dialogue_widget(
                    &mut scene,
                    &root,
                    &registry,
                    &model,
                    0,
                    presentation,
                )
                .unwrap();
                let defaults =
                    crate::dialogue_starter::widget_preset(presentation, display).unwrap();
                for rect in [
                    defaults.layout.panel,
                    defaults.layout.body,
                    defaults.layout.choice_panel,
                    defaults.layout.choices,
                    defaults.layout.name,
                ] {
                    assert!(
                        rect[0] + rect[2] <= display[0] && rect[1] + rect[3] <= display[1],
                        "{presentation:?}: {rect:?}"
                    );
                }
                let index = assets::scan(&root, &mut Default::default());
                let selection = Selection {
                    assets: BTreeMap::from([(conversation, assets::Kind::Dialogue)]),
                    pairs: BTreeSet::from([(conversation, Uuid::nil())]),
                };
                let bundle = Bundle::prepare_widgets(
                    &selection,
                    &index,
                    &registry,
                    display,
                    &Default::default(),
                    &[scene],
                )
                .unwrap();
                assert_eq!(bundle.variants.len(), 1);
                assert!(bundle.variants[0].widget.is_some());
                assert!(bundle.variants[0].texts.iter().any(|t| t.pages.len() > 1));
                assert!(
                    bundle.variants[0]
                        .texts
                        .iter()
                        .flat_map(|t| &t.pages)
                        .all(|p| p.text.len() <= 511)
                );
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
