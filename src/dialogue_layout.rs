//! Dialogue pagination uses ordinary authored UI controls and the HUD layout core.
//! A profile names geometry/fonts only; content and behavior stay in the graph.
use crate::{
    blueprint::Registry,
    dialogue_compile as graph,
    dialogue_cook::{Locales, Text},
    dialogue_text::{self, Metrics},
    scene::Scene,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub const PRESENTER: &str = "d40dc688-f7c0-50a3-9af8-7beaf1e3eea3";
pub const FIELDS: [&str; 11] = [
    "panel",
    "body",
    "speaker_name",
    "portrait",
    "choice0",
    "choice1",
    "choice2",
    "choice3",
    "prompt",
    "continuation",
    "choice_panel",
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Label {
    pub size: [u16; 2],
    pub font: Option<Uuid>,
    pub wrap: bool,
}
impl Label {
    fn metrics(
        &self,
        fonts: &BTreeMap<Uuid, std::sync::Arc<crate::font_asset::FontData>>,
    ) -> Result<Metrics, String> {
        self.font.map_or_else(
            || Ok(Metrics::builtin()),
            |id| {
                fonts
                    .get(&id)
                    .map(|font| Metrics::imported(font))
                    .ok_or_else(|| format!("Unstaged dialogue widget font {id}"))
            },
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Profile {
    pub body: Label,
    pub prompt: Option<Label>,
    pub name: Option<Label>,
    pub choices: Vec<Label>,
}
#[derive(Default)]
pub struct Collection {
    pub profiles: Vec<Profile>,
    /// Exact authored source/widget associations. Other supported combinations
    /// may still be cooked for runtime rebinding, without rejecting unrelated
    /// widgets merely because a label in another conversation does not fit.
    pub bound: BTreeSet<(Uuid, Uuid, u64)>,
}
pub fn property<'a>(
    registry: &'a Registry,
    class: &crate::actor_document::ClassReference,
    values: &'a BTreeMap<String, Value>,
    name: &str,
) -> Option<&'a Value> {
    values.get(name).or_else(|| {
        let class = class
            .class_id
            .as_ref()
            .and_then(|id| registry.classes.get(id))
            .or_else(|| registry.named(&class.name))?;
        registry.default_value(&class.cpp_name, name)
    })
}
pub fn reference(value: Option<&Value>) -> Option<Uuid> {
    value
        .and_then(Value::as_str)
        .and_then(|s| Uuid::parse_str(s).ok())
}
pub fn bindings(scene: &Scene, registry: &Registry, index: usize) -> BTreeMap<&'static str, usize> {
    let owner = &scene.actors[index];
    FIELDS
        .into_iter()
        .filter_map(|field| {
            let id = reference(property(registry, &owner.class, &owner.properties, field))?;
            let index = scene
                .actors
                .iter()
                .position(|a| a.components.iter().any(|c| c.id == id))?;
            Some((field, index))
        })
        .collect()
}
impl Profile {
    pub fn key(&self) -> u64 {
        let hash = crate::scene_dependencies::hash(self);
        u64::from_str_radix(&hash[..16], 16).unwrap().max(1)
    }
    pub fn read(scene: &Scene, registry: &Registry, index: usize) -> Result<Option<Self>, String> {
        let owner = &scene.actors[index];
        let model = registry.model().map_err(|errors| {
            errors
                .into_iter()
                .map(|e| e.message)
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        for field in FIELDS {
            let Some(id) = reference(property(registry, &owner.class, &owner.properties, field))
            else {
                continue;
            };
            let expected = if matches!(field, "panel" | "portrait" | "choice_panel") {
                crate::object_model::IMAGE_COMPONENT_ID
            } else {
                crate::object_model::TEXT_COMPONENT_ID
            };
            let component = scene
                .actors
                .iter()
                .flat_map(|a| &a.components)
                .find(|c| c.id == id);
            if component.is_none_or(|c| {
                !model.is_a(
                    c.class.class_id.as_deref().unwrap_or(&c.class.name),
                    expected,
                )
            }) {
                return Err(format!(
                    "Dialogue widget {}: {field} must reference an existing {} component",
                    owner.name,
                    if matches!(field, "panel" | "portrait" | "choice_panel") {
                        "Image"
                    } else {
                        "Text"
                    }
                ));
            }
        }
        let labels = bindings(scene, registry, index);
        let Some(body) = labels.get("body") else {
            return Ok(None);
        };
        let boxes = crate::hud::layouts(scene);
        let label = |index: usize| -> Result<Label, String> {
            let actor = &scene.actors[index];
            let text = actor.text.as_ref().ok_or_else(|| {
                format!(
                    "Dialogue widget {}: {} is not a Text control",
                    scene.actors[index].name, actor.name
                )
            })?;
            let rect = boxes
                .get(index)
                .copied()
                .flatten()
                .ok_or_else(|| {
                    format!(
                        "Dialogue widget control {} has no resolved RectTransform",
                        actor.name
                    )
                })?
                .rect;
            if !rect[2].is_finite()
                || !rect[3].is_finite()
                || rect[2] < 1.
                || rect[3] < 1.
                || rect[2] > u16::MAX as f32
                || rect[3] > u16::MAX as f32
            {
                return Err(format!(
                    "Dialogue widget control {} requires a positive bounded pixel area",
                    actor.name
                ));
            }
            Ok(Label {
                size: [rect[2].floor() as u16, rect[3].floor() as u16],
                font: text.font,
                wrap: text.wrap,
            })
        };
        let mut choices = vec![];
        for field in ["choice0", "choice1", "choice2", "choice3"] {
            if let Some(index) = labels.get(field) {
                choices.push(label(*index)?);
            }
        }
        Ok(Some(Self {
            body: label(*body)?,
            prompt: labels
                .get("prompt")
                .map(|index| label(*index))
                .transpose()?,
            name: labels
                .get("speaker_name")
                .map(|index| label(*index))
                .transpose()?,
            choices,
        }))
    }
    pub fn collect(scenes: &[Scene], registry: &Registry) -> Result<Collection, String> {
        let model = registry.model().map_err(|errors| {
            errors
                .into_iter()
                .map(|e| e.message)
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        let mut result: BTreeMap<u64, Self> = BTreeMap::new();
        for scene in scenes {
            for (index, actor) in scene.actors.iter().enumerate() {
                if !model.is_a(
                    actor.class.class_id.as_deref().unwrap_or(&actor.class.name),
                    PRESENTER,
                ) {
                    continue;
                }
                if let Some(profile) = Self::read(scene, registry, index)? {
                    let key = profile.key();
                    if result.get(&key).is_some_and(|other| other != &profile) {
                        return Err("Dialogue widget layout identity collision".into());
                    }
                    result.insert(key, profile);
                }
            }
        }
        let mut bound = BTreeSet::new();
        for scene in scenes {
            for actor in &scene.actors {
                for component in &actor.components {
                    if !model.is_a(
                        component
                            .class
                            .class_id
                            .as_deref()
                            .unwrap_or(&component.class.name),
                        "aa343ddf-5098-5b32-b993-d8b9b619d455",
                    ) {
                        continue;
                    }
                    let get = |name| {
                        reference(property(
                            registry,
                            &component.class,
                            &component.properties,
                            name,
                        ))
                    };
                    let Some(conversation) = get("conversation") else {
                        continue;
                    };
                    let Some(index) = get("presenter")
                        .and_then(|id| scene.actors.iter().position(|a| a.id == id))
                    else {
                        continue;
                    };
                    if let Some(profile) = Self::read(scene, registry, index)? {
                        bound.insert((
                            conversation,
                            get("style").unwrap_or(Uuid::nil()),
                            profile.key(),
                        ));
                    }
                }
            }
        }
        Ok(Collection {
            profiles: result.into_values().collect(),
            bound,
        })
    }
    pub fn supports(&self, graph: &graph::Compiled) -> bool {
        self.choices.len() == 4
            || !graph
                .nodes
                .iter()
                .any(|node| matches!(node.kind, graph::NodeKind::Choice { .. }))
    }
    pub fn texts(
        &self,
        graph: &graph::Compiled,
        locales: &Locales,
        fonts: &BTreeMap<Uuid, std::sync::Arc<crate::font_asset::FontData>>,
    ) -> Result<Vec<Text>, String> {
        let mut targets = vec![Vec::<&Label>::new(); graph.texts.len()];
        let mut single = vec![false; graph.texts.len()];
        for role in &graph.roles {
            if let Some(text) = role.display_name {
                single[usize::from(text)] = true;
                if let Some(name) = self.name.as_ref() {
                    targets[usize::from(text)].push(name);
                }
            }
        }
        for node in &graph.nodes {
            match &node.kind {
                graph::NodeKind::Line { text, .. } => targets[usize::from(*text)].push(&self.body),
                graph::NodeKind::Choice {
                    prompt, options, ..
                } => {
                    if let Some(prompt) = prompt {
                        targets[usize::from(*prompt)]
                            .push(self.prompt.as_ref().unwrap_or(&self.body));
                    }
                    if self.choices.len() != 4 {
                        return Err(
                            "Dialogue choice nodes require four bound choice Text controls".into(),
                        );
                    }
                    for option in options {
                        single[usize::from(option.text)] = true;
                        for choice in &self.choices {
                            targets[usize::from(option.text)].push(choice);
                        }
                    }
                }
                _ => {}
            }
        }
        let mut result = vec![];
        for (index, key) in graph.texts.iter().enumerate() {
            let labels = &targets[index];
            let mut value = Text {
                pages: vec![],
                offsets: vec![0],
            };
            for (language, name) in locales.names.iter().enumerate() {
                let context = |e: String| {
                    format!(
                        "Dialogue widget / {} / {} / {name}: {e}",
                        key.table, key.key
                    )
                };
                let raw = locales.text(key, language)?;
                let pages = if single[index] {
                    if raw.len() > dialogue_text::MAX_PAGE_BYTES {
                        return Err(context(
                            "single labels exceed the 511-byte Text payload".into(),
                        ));
                    }
                    // An unbound optional name is data, not a rendered label.
                    // It must remain bounded without imposing a builtin font.
                    let mut glyphs = crate::hud::visible_text(raw)
                        .chars()
                        .filter(|c| *c != '\n')
                        .count() as u16;
                    let mut checked = BTreeSet::new();
                    for label in labels {
                        if !checked.insert((label.size, label.font, label.wrap)) {
                            continue;
                        }
                        let pages = dialogue_text::paginate_control(
                            raw,
                            &label.metrics(fonts)?,
                            label.size[0],
                            label.size[1],
                            label.wrap,
                        )
                        .map_err(context)?;
                        if pages.len() != 1 {
                            return Err(context(
                                "choice or participant name does not fit its authored Text control"
                                    .into(),
                            ));
                        }
                        glyphs = pages[0].glyphs;
                    }
                    // Single labels keep the source's line breaks. Their own
                    // fonts/wrapping may differ; validation covered every row.
                    vec![dialogue_text::Page {
                        text: raw.into(),
                        glyphs,
                    }]
                } else {
                    let label = labels.first().copied().unwrap_or(&self.body);
                    dialogue_text::paginate_control(
                        raw,
                        &label.metrics(fonts)?,
                        label.size[0],
                        label.size[1],
                        label.wrap,
                    )
                    .map_err(context)?
                };
                value.pages.extend(pages);
                value.offsets.push(
                    u16::try_from(value.pages.len())
                        .map_err(|_| context("combined translations exceed 65535 pages".into()))?,
                );
            }
            result.push(value);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assets, dialogue_asset as da,
        dialogue_cook::{Bundle, Selection},
        localization::{self, Entry, StringTable},
    };
    fn font() -> crate::font_asset::FontData {
        crate::font_asset::FontData {
            width: 64,
            height: 16,
            words: vec![0; 256],
            palette: [0; 16],
            metrics: [('W', 7), ('i', 1)]
                .into_iter()
                .map(|(c, advance)| crate::font_asset::GlyphMetric {
                    codepoint: c as u32,
                    u: 0,
                    v: 0,
                    width: advance,
                    height: 8,
                    advance,
                    x_offset: 0,
                    y_offset: 0,
                })
                .collect(),
            line_height: 8,
            baseline: 8,
            rgba: vec![0; 64 * 16 * 4],
        }
    }
    #[test]
    fn authored_widgets_get_independent_pages_fonts_and_shared_label_text_without_style_geometry() {
        let root = crate::workspace::tests::temp("dialogue-widget-layouts");
        crate::workspace::create(&root, "Widget Layouts", crate::workspace::Template::Basic)
            .unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let table = localization::create(
            &root,
            "assets/Words.epokasset",
            &StringTable {
                pageable: true,
                languages: vec!["English".into(), "French".into()],
                entries: vec![Entry {
                    key: "shared".into(),
                    values: vec!["WiWiWi".into(), "iiiiii".into()],
                }],
                ..Default::default()
            },
        )
        .unwrap();
        let text = da::TextKey {
            table,
            key: "shared".into(),
        };
        let role = Uuid::new_v4();
        let line = Uuid::new_v4();
        let choice = Uuid::new_v4();
        let end = Uuid::new_v4();
        let source = da::DialogueAsset {
            version: da::VERSION,
            entry: line,
            participants: vec![da::Participant {
                id: role,
                label: "Speaker".into(),
                display_name: Some(text.clone()),
                portrait: None,
            }],
            bindings: vec![],
            nodes: vec![
                da::Node {
                    id: line,
                    kind: da::NodeKind::Line {
                        speaker: Some(role),
                        text: text.clone(),
                        next: choice,
                        policy: None,
                    },
                },
                da::Node {
                    id: choice,
                    kind: da::NodeKind::Choice {
                        speaker: Some(role),
                        prompt: Some(text.clone()),
                        options: vec![da::ChoiceOption {
                            id: Uuid::new_v4(),
                            text: text.clone(),
                            next: end,
                            condition: None,
                        }],
                    },
                },
                da::Node {
                    id: end,
                    kind: da::NodeKind::End,
                },
            ],
        };
        let conversation = da::create(&root, "assets/Talk.epokasset", &source).unwrap();
        let mut first = Scene::default();
        first.actors.clear();
        let root_index = crate::editor::place_class(
            &mut first,
            &root,
            &registry,
            &registry.model().unwrap(),
            PRESENTER,
        )
        .unwrap();
        let labels = bindings(&first, &registry, root_index);
        let font_id = Uuid::new_v4();
        let fonts = BTreeMap::from([(font_id, std::sync::Arc::new(font()))]);
        let body = labels["body"];
        first.actors[body].rect.as_mut().unwrap().anchor_max = [0.; 2];
        first.actors[body].rect.as_mut().unwrap().size = [8., 8.];
        first.actors[body].text.as_mut().unwrap().font = Some(font_id);
        let prompt = labels["prompt"];
        first.actors[prompt].rect.as_mut().unwrap().anchor_max = [0.; 2];
        first.actors[prompt].rect.as_mut().unwrap().size = [16., 8.];
        first.actors[prompt].text.as_mut().unwrap().font = Some(font_id);
        first.actors[labels["speaker_name"]]
            .rect
            .as_mut()
            .unwrap()
            .anchor_max = [0.; 2];
        first.actors[labels["speaker_name"]]
            .rect
            .as_mut()
            .unwrap()
            .size = [64., 16.];
        first.fonts = fonts.clone();
        first.sync_actor_components();
        let mut second = first.clone();
        second.name = "Other layout".into();
        second.actors[body].rect.as_mut().unwrap().size = [16., 8.];
        second.actors[prompt].rect.as_mut().unwrap().size = [24., 8.];
        second.sync_actor_components();
        let a = Profile::read(&first, &registry, root_index)
            .unwrap()
            .unwrap();
        let b = Profile::read(&second, &registry, root_index)
            .unwrap()
            .unwrap();
        assert_ne!(a.key(), b.key());
        let index = assets::scan(&root, &mut Default::default());
        let selection = Selection {
            assets: BTreeMap::from([
                (conversation, assets::Kind::Dialogue),
                (table, assets::Kind::StringTable),
            ]),
            pairs: BTreeSet::from([(conversation, Uuid::nil())]),
        };
        let mut unrelated = first.clone();
        let row = labels["choice0"];
        unrelated.actors[row].rect.as_mut().unwrap().anchor_min = [0.; 2];
        unrelated.actors[row].rect.as_mut().unwrap().anchor_max = [0.; 2];
        unrelated.actors[row].rect.as_mut().unwrap().size = [8., 16.];
        let bundle = Bundle::prepare_widgets(
            &selection,
            &index,
            &registry,
            [320, 240],
            &fonts,
            &[first.clone(), second, unrelated.clone()],
        )
        .unwrap();
        assert_eq!(bundle.variants.len(), 2);
        for (profile, pages) in [(&a, 3), (&b, 2)] {
            let variant = bundle
                .variants
                .iter()
                .find(|v| v.widget.as_ref() == Some(profile))
                .unwrap();
            let body = variant
                .graph
                .nodes
                .iter()
                .find_map(|n| {
                    if let graph::NodeKind::Line { text, .. } = n.kind {
                        Some(text)
                    } else {
                        None
                    }
                })
                .unwrap();
            let name = variant.graph.roles[0].display_name.unwrap();
            let prompt = variant
                .graph
                .nodes
                .iter()
                .find_map(|n| {
                    if let graph::NodeKind::Choice { prompt, .. } = n.kind {
                        prompt
                    } else {
                        None
                    }
                })
                .unwrap();
            assert_ne!(body, name);
            assert_ne!(body, prompt);
            assert_ne!(name, prompt);
            let prompt_pages = pages - 1;
            assert_eq!(
                variant.texts[usize::from(prompt)].offsets,
                [0, prompt_pages, prompt_pages + 1]
            );
            assert_eq!(
                variant.texts[usize::from(body)].offsets,
                [0, pages, pages + 1]
            );
            assert_eq!(variant.texts[usize::from(name)].pages[0].text, "WiWiWi");
            assert_eq!(variant.texts[usize::from(name)].offsets, [0, 1, 2]);
        }
        let header = bundle.header(&first).unwrap();
        assert!(
            header.contains(&format!("UINT64_C({})", a.key()))
                && header.contains(&format!("UINT64_C({})", b.key()))
        );
        crate::texture::resolve(&mut first, &index).unwrap();
        let scene_header = crate::project::scene_header_with_registry(
            &first,
            &[],
            &first,
            true,
            &first,
            &registry,
        )
        .unwrap();
        assert!(scene_header.contains(&format!("bind_widget_layout(actor,UINT64_C({}))", a.key())));
        // An unrelated widget with a narrower choice row is not a constraint
        // on this conversation. Binding it deliberately produces a diagnostic.
        let source = crate::editor::place_class(
            &mut unrelated,
            &root,
            &registry,
            &registry.model().unwrap(),
            crate::object_model::ACTOR3D_ID,
        )
        .unwrap();
        let mut component = crate::actor_document::ComponentInstance::new(
            Uuid::new_v4(),
            crate::actor_document::ClassReference::new(
                "epok::DialogueInteractionComponent",
                "aa343ddf-5098-5b32-b993-d8b9b619d455",
            ),
            "Dialogue",
        );
        component
            .properties
            .insert("conversation".into(), serde_json::json!(conversation));
        component.properties.insert(
            "presenter".into(),
            serde_json::json!(unrelated.actors[root_index].id),
        );
        unrelated.actors[source].components.push(component);
        assert!(
            Bundle::prepare_widgets(
                &selection,
                &index,
                &registry,
                [320, 240],
                &fonts,
                &[first.clone(), unrelated]
            )
            .unwrap_err()
            .contains("does not fit")
        );
        // Position/color are authored appearance, not pagination identities.
        first.actors[body].rect.as_mut().unwrap().position = [123., 42.];
        first.actors[body].text.as_mut().unwrap().color = [0.2, 0.4, 0.7];
        assert_eq!(
            Profile::read(&first, &registry, root_index)
                .unwrap()
                .unwrap()
                .key(),
            a.key()
        );
        let wrong_type = first.actors[root_index].properties["panel"].clone();
        first.actors[root_index]
            .properties
            .insert("body".into(), wrong_type);
        assert!(
            Profile::read(&first, &registry, root_index)
                .unwrap_err()
                .contains("Text component")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn dialogue_unbound_names_remain_bounded_without_imposing_a_font() {
        let root = crate::workspace::tests::temp("dialogue-optional-name");
        crate::workspace::create(&root, "Optional Name", crate::workspace::Template::Basic)
            .unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let table = localization::create(
            &root,
            "assets/Names.epokasset",
            &StringTable {
                pageable: true,
                entries: vec![
                    Entry {
                        key: "words".into(),
                        values: vec!["Wi".into()],
                    },
                    Entry {
                        key: "name".into(),
                        values: vec!["桜".into()],
                    },
                ],
                ..Default::default()
            },
        )
        .unwrap();
        let line = Uuid::new_v4();
        let end = Uuid::new_v4();
        let role = Uuid::new_v4();
        let index = assets::scan(&root, &mut Default::default());
        let graph = graph::compile(
            Uuid::new_v4(),
            &da::DialogueAsset {
                version: da::VERSION,
                entry: line,
                participants: vec![da::Participant {
                    id: role,
                    label: "Name".into(),
                    display_name: Some(da::TextKey {
                        table,
                        key: "name".into(),
                    }),
                    portrait: None,
                }],
                bindings: vec![],
                nodes: vec![
                    da::Node {
                        id: line,
                        kind: da::NodeKind::Line {
                            speaker: Some(role),
                            text: da::TextKey {
                                table,
                                key: "words".into(),
                            },
                            next: end,
                            policy: None,
                        },
                    },
                    da::Node {
                        id: end,
                        kind: da::NodeKind::End,
                    },
                ],
            },
            &registry,
            &index,
        )
        .unwrap();
        let font_id = Uuid::new_v4();
        let fonts = BTreeMap::from([(font_id, std::sync::Arc::new(font()))]);
        let label = Label {
            size: [8, 8],
            font: Some(font_id),
            wrap: true,
        };
        let mut profile = Profile {
            body: label.clone(),
            prompt: None,
            name: None,
            choices: vec![],
        };
        let locales = Locales::prepare(&index, &BTreeSet::from([table])).unwrap();
        let name = usize::from(graph.roles[0].display_name.unwrap());
        let texts = profile.texts(&graph, &locales, &fonts).unwrap();
        assert_eq!(texts[name].pages[0].text, "桜");
        assert_eq!(texts[name].pages[0].glyphs, 1);
        profile.name = Some(label);
        assert!(
            profile
                .texts(&graph, &locales, &fonts)
                .unwrap_err()
                .contains("no glyph")
        );
        profile.name = None;
        let record = index.resolve(table).unwrap();
        let mut values = localization::document(record).unwrap();
        values
            .entries
            .iter_mut()
            .find(|e| e.key == "name")
            .unwrap()
            .values[0] = "x".repeat(512);
        localization::save(record, &values).unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let locales = Locales::prepare(&index, &BTreeSet::from([table])).unwrap();
        assert!(
            profile
                .texts(&graph, &locales, &fonts)
                .unwrap_err()
                .contains("511-byte")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
