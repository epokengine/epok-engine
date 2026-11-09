//! Resolve dialogue graph identities and reflected callback signatures on the host.
//! Runtime tables contain compact indices and typed dispatch, not expressions.
#![allow(dead_code)] // Source identities remain available to P4 authoring diagnostics.
use crate::{blueprint::Registry, dialogue_asset as asset, reflection_schema as schema};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TextUse {
    Line,
    Prompt,
    Label,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Role {
    pub id: u32,
    pub source: Uuid,
    pub display_name: Option<u16>,
    pub portrait: Option<Uuid>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    pub id: u32,
    pub source: Uuid,
    pub role: u8,
    pub kind: asset::BindingKind,
    pub class_id: String,
    pub cpp_class: String,
    pub cpp_method: String,
    pub member_id: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct OptionRow {
    pub id: u32,
    pub text: u16,
    pub next: u16,
    pub condition: Option<u8>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum NodeKind {
    Line {
        speaker: Option<u8>,
        text: u16,
        next: u16,
        policy: Option<asset::LinePolicy>,
    },
    Choice {
        speaker: Option<u8>,
        prompt: Option<u16>,
        options: Vec<OptionRow>,
    },
    Branch {
        condition: u8,
        yes: u16,
        no: u16,
    },
    Event {
        action: u8,
        next: u16,
    },
    End,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: u32,
    pub source: Uuid,
    pub kind: NodeKind,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Compiled {
    pub asset: Uuid,
    pub entry: u16,
    pub roles: Vec<Role>,
    pub bindings: Vec<Binding>,
    pub texts: Vec<asset::TextKey>,
    pub nodes: Vec<Node>,
}

/// Zero is reserved for an unassigned role. Collisions are diagnosed at cook.
pub fn compact(id: Uuid) -> u32 {
    u32::from_le_bytes(Sha256::digest(id.as_bytes())[..4].try_into().unwrap())
}
fn identities(source: &asset::DialogueAsset) -> Result<(), String> {
    let mut used = BTreeMap::new();
    let options = source.nodes.iter().flat_map(|n| match &n.kind {
        asset::NodeKind::Choice { options, .. } => options.iter().map(|o| o.id).collect::<Vec<_>>(),
        _ => Vec::new(),
    });
    for id in source
        .participants
        .iter()
        .map(|r| r.id)
        .chain(source.bindings.iter().map(|b| b.id))
        .chain(source.nodes.iter().map(|n| n.id))
        .chain(options)
    {
        let key = compact(id);
        if key == 0 || used.insert(key, id).is_some_and(|other| other != id) {
            return Err(format!(
                "Dialogue compact identity collision at {id}; assign a fresh identity before cooking"
            ));
        }
    }
    Ok(())
}

fn callback(binding: &asset::Binding, registry: &Registry) -> Result<(String, String), String> {
    let context = || format!("Dialogue binding '{}' ({})", binding.label, binding.id);
    let class = registry
        .classes
        .get(&binding.class)
        .ok_or_else(|| format!("{}: missing class identity {}", context(), binding.class))?;
    let model = registry.model().map_err(|errors| {
        errors
            .into_iter()
            .map(|e| e.message)
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let resolved = model
        .class(&class.id)
        .ok_or_else(|| format!("{}: class has no runtime object representation", context()))?;
    if !matches!(
        resolved.family,
        schema::ClassFamily::Actor | schema::ClassFamily::Component
    ) || class.backend != schema::native_backend()
    {
        return Err(format!(
            "{}: receiver must be an Actor or ActorComponent with a native runtime representation",
            context()
        ));
    }
    let mut methods = BTreeMap::new();
    for ancestor in registry.ancestry(&class.cpp_name) {
        for method in &ancestor.functions {
            for id in std::iter::once(&method.id).chain(&method.overrides) {
                methods.insert(id, method);
            }
        }
    }
    let method = methods.get(&binding.member).ok_or_else(|| {
        format!(
            "{}: missing reflected member identity {}",
            context(),
            binding.member
        )
    })?;
    if method.access != "public" || !(method.callable || method.event) {
        return Err(format!(
            "{}: callback must be a public reflected callable or event",
            context()
        ));
    }
    if !callback_signature(binding.kind, method) {
        return Err(format!(
            "{}: expected {}, found {}",
            context(),
            match binding.kind {
                asset::BindingKind::Condition => "pure const bool(ObjectId initiator)",
                asset::BindingKind::Action => "void(ObjectId initiator, uint32_t port)",
            },
            method.name
        ));
    }
    Ok((class.cpp_name.clone(), method.name.clone()))
}

/// The authoring picker and cook accept the same synchronous callback shape.
pub fn callback_signature(kind: asset::BindingKind, method: &schema::Function) -> bool {
    if method.access != "public" || !(method.callable || method.event) {
        return false;
    }
    let initiator = schema::Type::ObjectRef { class: None };
    let parameters = method
        .parameters
        .iter()
        .map(|p| (&p.value_type, &p.direction))
        .collect::<Vec<_>>();
    match kind {
        asset::BindingKind::Condition => {
            method.pure
                && method.returns == schema::Type::Bool
                && parameters == vec![(&initiator, &schema::Direction::Value)]
        }
        asset::BindingKind::Action => {
            method.returns == schema::Type::Void
                && parameters
                    == vec![
                        (&initiator, &schema::Direction::Value),
                        (&schema::Type::UInt32, &schema::Direction::Value),
                    ]
        }
    }
}

pub fn compile(
    id: Uuid,
    source: &asset::DialogueAsset,
    registry: &Registry,
    index: &crate::assets::Index,
) -> Result<Compiled, String> {
    if id.is_nil() {
        return Err("Dialogue needs a persistent asset UUID".into());
    }
    source.validate_resources(index)?;
    identities(source)?;
    let mut roles = source.participants.iter().collect::<Vec<_>>();
    roles.sort_by_key(|r| r.id);
    let mut bindings = source.bindings.iter().collect::<Vec<_>>();
    bindings.sort_by_key(|b| b.id);
    let mut nodes = source.nodes.iter().collect::<Vec<_>>();
    nodes.sort_by_key(|n| n.id);
    // A body can span pages while a name/option must fit its own single label.
    // Reusing a localization key across those uses must not couple their pages.
    let mut text_uses = BTreeSet::new();
    for role in &roles {
        if let Some(key) = &role.display_name {
            text_uses.insert((key.clone(), TextUse::Label));
        }
    }
    for node in &nodes {
        match &node.kind {
            asset::NodeKind::Line { text, .. } => {
                text_uses.insert((text.clone(), TextUse::Line));
            }
            asset::NodeKind::Choice {
                prompt, options, ..
            } => {
                if let Some(key) = prompt {
                    text_uses.insert((key.clone(), TextUse::Prompt));
                }
                for option in options {
                    text_uses.insert((option.text.clone(), TextUse::Label));
                }
            }
            _ => {}
        }
    }
    let text_uses = text_uses.into_iter().collect::<Vec<_>>();
    let texts = text_uses
        .iter()
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    if text_uses.len() > usize::from(u16::MAX) {
        return Err("Dialogue exceeds 65535 distinct localized text references".into());
    }
    let role_index = roles
        .iter()
        .enumerate()
        .map(|(i, r)| (r.id, i as u8))
        .collect::<BTreeMap<_, _>>();
    let binding_index = bindings
        .iter()
        .enumerate()
        .map(|(i, b)| (b.id, i as u8))
        .collect::<BTreeMap<_, _>>();
    let node_index = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id, i as u16))
        .collect::<BTreeMap<_, _>>();
    let text_index = text_uses
        .iter()
        .enumerate()
        .map(|(i, (t, label))| ((t, *label), i as u16))
        .collect::<BTreeMap<_, _>>();
    // References were checked before indexing; no missing-edge fallback is emitted.
    let cooked_roles = roles
        .into_iter()
        .map(|r| Role {
            id: compact(r.id),
            source: r.id,
            display_name: r
                .display_name
                .as_ref()
                .map(|t| text_index[&(t, TextUse::Label)]),
            portrait: r.portrait,
        })
        .collect();
    let cooked_bindings = bindings
        .into_iter()
        .map(|b| {
            let (cpp_class, cpp_method) = callback(b, registry)?;
            Ok(Binding {
                id: compact(b.id),
                source: b.id,
                role: role_index[&b.participant],
                kind: b.kind,
                class_id: b.class.clone(),
                cpp_class,
                cpp_method,
                member_id: b.member.clone(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let cooked_nodes = nodes
        .into_iter()
        .map(|n| Node {
            id: compact(n.id),
            source: n.id,
            kind: match &n.kind {
                asset::NodeKind::Line {
                    speaker,
                    text,
                    next,
                    policy,
                } => NodeKind::Line {
                    speaker: speaker.map(|r| role_index[&r]),
                    text: text_index[&(text, TextUse::Line)],
                    next: node_index[next],
                    policy: policy.clone(),
                },
                asset::NodeKind::Choice {
                    speaker,
                    prompt,
                    options,
                } => NodeKind::Choice {
                    speaker: speaker.map(|r| role_index[&r]),
                    prompt: prompt.as_ref().map(|t| text_index[&(t, TextUse::Prompt)]),
                    options: options
                        .iter()
                        .map(|o| OptionRow {
                            id: compact(o.id),
                            text: text_index[&(&o.text, TextUse::Label)],
                            next: node_index[&o.next],
                            condition: o.condition.map(|b| binding_index[&b]),
                        })
                        .collect(),
                },
                asset::NodeKind::Branch { condition, yes, no } => NodeKind::Branch {
                    condition: binding_index[condition],
                    yes: node_index[yes],
                    no: node_index[no],
                },
                asset::NodeKind::Event { action, next } => NodeKind::Event {
                    action: binding_index[action],
                    next: node_index[next],
                },
                asset::NodeKind::End => NodeKind::End,
            },
        })
        .collect();
    Ok(Compiled {
        asset: id,
        entry: node_index[&source.entry],
        roles: cooked_roles,
        bindings: cooked_bindings,
        texts,
        nodes: cooked_nodes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialogue_callbacks_use_reflected_signatures_and_ignore_document_order() {
        let root = crate::workspace::tests::temp("dialogue-compile");
        crate::workspace::create(&root, "Dialogue Compile", crate::workspace::Template::Basic)
            .unwrap();
        let registry = crate::blueprint::native_registry(&root, &[]).unwrap();
        let role = Uuid::new_v4();
        let condition = Uuid::new_v4();
        let action = Uuid::new_v4();
        let entry = Uuid::new_v4();
        let event = Uuid::new_v4();
        let end = Uuid::new_v4();
        let id = Uuid::new_v4();
        let mut source = asset::DialogueAsset {
            version: asset::VERSION,
            entry,
            participants: vec![asset::Participant {
                id: role,
                label: "Rules".into(),
                display_name: None,
                portrait: None,
            }],
            bindings: vec![
                asset::Binding {
                    id: condition,
                    label: "Available".into(),
                    participant: role,
                    kind: asset::BindingKind::Condition,
                    class: "310e882c-2353-55bc-bba3-f3d54277f671".into(),
                    member: "fb9c1c09-be15-5b42-a069-4480e8e389f2".into(),
                },
                asset::Binding {
                    id: action,
                    label: "Accepted".into(),
                    participant: role,
                    kind: asset::BindingKind::Action,
                    class: "310e882c-2353-55bc-bba3-f3d54277f671".into(),
                    member: "e66c2703-4f26-5bfc-b269-2f0c0eefbb8f".into(),
                },
            ],
            nodes: vec![
                asset::Node {
                    id: entry,
                    kind: asset::NodeKind::Branch {
                        condition,
                        yes: event,
                        no: end,
                    },
                },
                asset::Node {
                    id: event,
                    kind: asset::NodeKind::Event { action, next: end },
                },
                asset::Node {
                    id: end,
                    kind: asset::NodeKind::End,
                },
            ],
        };
        let index = crate::assets::Index::default();
        let first = compile(id, &source, &registry, &index).unwrap();
        assert!(
            first
                .bindings
                .iter()
                .any(|b| b.cpp_method == "can_interact")
        );
        assert!(
            first
                .bindings
                .iter()
                .any(|b| b.cpp_method == "interaction_started")
        );
        source.nodes.reverse();
        source.bindings.reverse();
        assert_eq!(compile(id, &source, &registry, &index).unwrap(), first);
        source
            .bindings
            .iter_mut()
            .find(|b| b.kind == asset::BindingKind::Condition)
            .unwrap()
            .member = "7d166263-f599-5f79-8f93-3ba11dc3a70d".into();
        let error = compile(id, &source, &registry, &index).unwrap_err();
        assert!(
            error.contains("Available") && error.contains("pure const bool"),
            "{error}"
        );
        source
            .bindings
            .iter_mut()
            .find(|b| b.kind == asset::BindingKind::Condition)
            .unwrap()
            .member = "missing-member".into();
        assert!(
            compile(id, &source, &registry, &index)
                .unwrap_err()
                .contains("missing reflected member")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
