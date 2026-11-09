//! Validate real dialogue widget states with the native HUD compiler. Binding
//! changes content/visibility only; the authored controls own their appearance.
use crate::{
    blueprint::Registry,
    dialogue_compile::NodeKind,
    dialogue_cook::{Bundle, Variant},
    dialogue_layout::{self, property, reference},
    scene::Scene,
};
use std::collections::BTreeMap;
use uuid::Uuid;

fn glyphs(text: &str) -> usize {
    crate::hud::visible_text(text)
        .chars()
        .filter(|c| !c.is_whitespace())
        .count()
}
fn text(scene: &mut Scene, index: usize, value: &str) {
    if let Some(text) = scene.actors[index].text.as_mut() {
        text.enabled = true;
        text.text = value.into();
        text.key.clear();
    }
}
fn portrait(scene: &mut Scene, index: usize, texture: Uuid) {
    if let Some(image) = scene.actors[index].image.as_mut() {
        image.enabled = true;
        image.texture = Some(texture);
    }
}
/// Deterministic sample data for the normal widget designer. This does not run
/// project callbacks or write content, layout or visibility back to the class.
pub fn sample(scene: &mut Scene, registry: &Registry, state: usize) {
    let roots = scene
        .actors
        .iter()
        .enumerate()
        .filter_map(|(i, a)| {
            registry
                .ancestry(&a.class.name)
                .iter()
                .any(|c| c.id == dialogue_layout::PRESENTER)
                .then_some(i)
        })
        .collect::<Vec<_>>();
    for root in roots {
        let labels = dialogue_layout::bindings(scene, registry, root);
        for (field, index) in labels {
            if let Some(text) = scene.actors[index].text.as_mut() {
                if field == "continuation" {
                    text.enabled = state == 1 || state == 4;
                    continue;
                }
                let (visible, value) = match field {
                    "body" => (state == 1, "Dialogue preview."),
                    "prompt" => (state == 2 || state == 4, "How can I help?"),
                    "speaker_name" => (state == 1 || state == 2 || state == 4, "Speaker"),
                    "choice0" => (state == 2, "Tell me more."),
                    "choice1" => (state == 2, "Where are we?"),
                    "choice2" => (state == 2, "I can help."),
                    "choice3" => (state == 2, "Goodbye."),
                    _ => (false, ""),
                };
                text.enabled = visible;
                text.key.clear();
                text.text = value.into();
            }
            if let Some(image) = scene.actors[index].image.as_mut() {
                image.enabled = if field == "choice_panel" {
                    state == 2
                } else {
                    state == 1 || state == 2 || state == 4
                };
            }
            if field == "choice0"
                && state == 2
                && let Some(canvas) = scene.actors[root].canvas.as_mut()
            {
                canvas.focused = index as i32;
                canvas.initial_focus = None;
            }
        }
        if state != 2
            && let Some(canvas) = scene.actors[root].canvas.as_mut()
        {
            canvas.focused = -1;
            canvas.initial_focus = None;
        }
    }
}
fn base(
    scene: &Scene,
    resources: &Scene,
    labels: &BTreeMap<&str, usize>,
    variant: &Variant,
) -> Scene {
    let mut baseline = scene.clone();
    baseline.fonts = resources.fonts.clone();
    baseline.textures = resources.textures.clone();
    for index in labels.values() {
        if let Some(text) = baseline.actors[*index].text.as_mut() {
            text.enabled = false;
        }
        if let Some(image) = baseline.actors[*index].image.as_mut() {
            image.enabled = false;
        }
    }
    if let Some(panel) = labels.get("panel")
        && let Some(image) = baseline.actors[*panel].image.as_mut()
    {
        image.enabled = true;
    }
    if let Some(name) = labels.get("speaker_name")
        && let Some(value) = variant
            .graph
            .roles
            .iter()
            .filter_map(|r| r.display_name)
            .flat_map(|i| &variant.texts[usize::from(i)].pages)
            .max_by_key(|p| glyphs(&p.text))
    {
        text(&mut baseline, *name, &value.text);
    }
    if let Some(image) = labels.get("portrait")
        && let Some(id) = variant.graph.roles.iter().find_map(|r| r.portrait)
    {
        portrait(&mut baseline, *image, id);
    }
    baseline
}
pub fn validate(
    scenes: &[Scene],
    resources: &Scene,
    registry: &Registry,
    bundle: &Bundle,
) -> Result<(), String> {
    if bundle.variants.is_empty() {
        return Ok(());
    }
    let model = registry.model().map_err(|e| {
        e.into_iter()
            .map(|e| e.message)
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    for scene in scenes {
        for source in &scene.actors {
            for component in &source.components {
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
                let get = |name| property(registry, &component.class, &component.properties, name);
                let Some(conversation) = reference(get("conversation")) else {
                    continue;
                };
                let style = reference(get("style")).unwrap_or(Uuid::nil());
                let Some(presenter) = reference(get("presenter"))
                    .and_then(|id| scene.actors.iter().position(|a| a.id == id))
                else {
                    continue;
                };
                let labels = dialogue_layout::bindings(scene, registry, presenter);
                let Some(body) = labels.get("body").copied() else {
                    continue;
                };
                let profile = dialogue_layout::Profile::read(scene, registry, presenter)?;
                let widget = profile.as_ref().map_or(0, |p| p.key());
                let variant=bundle.variants.iter().find(|v|v.graph.asset==conversation && v.style.id==style && v.widget.as_ref().map_or(0,|p|p.key())==widget)
            .ok_or_else(||format!("Dialogue {} / {}: the bound widget has no compatible cooked presentation; choice nodes need four Text controls",source.name,component.name))?;
                let check = |profile: &Scene| -> Result<(), String> {
                    let (_, stats) = crate::hud_native::compile(profile);
                    if stats[4] > 0 {
                        return Err(format!(
                            "Dialogue HUD budget exceeded for {} / {} in scene {} ({} dropped primitives); reserve layouts, text, glyphs and panel rectangles for the shared presenter",
                            source.name, component.name, scene.name, stats[4]
                        ));
                    }
                    Ok(())
                };
                let common = base(scene, resources, &labels, variant);
                let continuation = labels.get("continuation").copied();
                for prompting in [false, true] {
                    let largest = variant
                        .graph
                        .nodes
                        .iter()
                        .filter_map(|node| match &node.kind {
                            NodeKind::Line { text, .. } if !prompting => Some(*text),
                            NodeKind::Choice { prompt, .. } if prompting => *prompt,
                            _ => None,
                        })
                        .flat_map(|i| &variant.texts[usize::from(i)].pages)
                        .max_by_key(|p| glyphs(&p.text));
                    if let Some(value) = largest {
                        let mut waiting = common.clone();
                        let target = if prompting {
                            labels.get("prompt").copied().unwrap_or(body)
                        } else {
                            body
                        };
                        text(&mut waiting, target, &value.text);
                        if let Some(index) = continuation
                            && let Some(text) = waiting.actors[index].text.as_mut()
                        {
                            text.enabled = true;
                        }
                        check(&waiting)?;
                    }
                }
                for node in &variant.graph.nodes {
                    let NodeKind::Choice {
                        prompt, options, ..
                    } = &node.kind
                    else {
                        continue;
                    };
                    // Conditions can expose any row; reserve the largest four labels
                    // without executing project predicates during discovery.
                    let mut values = options
                        .iter()
                        .map(|o| {
                            variant.texts[usize::from(o.text)]
                                .pages
                                .iter()
                                .max_by_key(|p| glyphs(&p.text))
                                .unwrap()
                        })
                        .collect::<Vec<_>>();
                    values.sort_by_key(|p| std::cmp::Reverse(glyphs(&p.text)));
                    let mut choice = common.clone();
                    if let Some(index) = labels.get("choice_panel") {
                        choice.actors[*index].image.as_mut().unwrap().enabled = true;
                    }
                    if let Some(prompt) = prompt
                        && let Some(index) = labels.get("prompt")
                    {
                        let value = &variant.texts[usize::from(*prompt)];
                        if let Some(last) = value
                            .offsets
                            .windows(2)
                            .filter(|range| range[1] > range[0])
                            .map(|range| &value.pages[usize::from(range[1] - 1)])
                            .max_by_key(|p| glyphs(&p.text))
                        {
                            text(&mut choice, *index, &last.text);
                        }
                    }
                    for (row, value) in values.into_iter().take(4).enumerate() {
                        if let Some(label) = labels.get(format!("choice{row}").as_str()) {
                            text(&mut choice, *label, &value.text);
                        }
                    }
                    check(&choice)?;
                    for field in ["choice0", "choice1", "choice2", "choice3"] {
                        if let Some(index) = labels.get(field)
                            && let Some(canvas) = choice.actors[presenter].canvas.as_mut()
                        {
                            canvas.focused = *index as i32;
                            canvas.initial_focus = None;
                            check(&choice)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
