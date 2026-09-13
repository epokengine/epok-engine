//! Split pins keep ordinary, typed links; dotted paths select aggregate members.
use super::*;

fn key(socket: &Socket) -> String {
    format!(
        "{}:{}:{}",
        socket.node,
        if socket.output { "out" } else { "in" },
        socket.pin
    )
}
fn split(doc: &BlueprintAsset, node: &Node, socket: &Socket) -> bool {
    doc.layout.split_pins.contains(&key(socket))
        || (!socket.output
            && node
                .inputs
                .keys()
                .any(|p| p.starts_with(&format!("{}.", socket.pin))))
}
pub(super) fn expand(doc: &BlueprintAsset, node: &Node, sockets: Vec<Socket>) -> Vec<Socket> {
    fn visit(
        doc: &BlueprintAsset,
        node: &Node,
        socket: Socket,
        result: &mut Vec<Socket>,
        depth: usize,
    ) {
        if depth < 16
            && split(doc, node, &socket)
            && let SocketType::Value(ty) = &socket.ty
            && !ty.members().is_empty()
        {
            for field in ty.members() {
                visit(
                    doc,
                    node,
                    Socket {
                        pin: format!("{}.{}", socket.pin, field.name),
                        label: format!("{} {}", socket.label, display_port(&field.name)),
                        ty: SocketType::Value(field.value_type),
                        ..socket.clone()
                    },
                    result,
                    depth + 1,
                );
            }
        } else {
            result.push(socket);
        }
    }
    let mut result = Vec::new();
    for socket in sockets {
        visit(doc, node, socket, &mut result, 0);
    }
    result
}
fn child_value(value: &Value, ty: &schema::Type, index: usize, name: &str) -> Option<Value> {
    if matches!(ty, schema::Type::Vector { .. }) {
        value.get(index).cloned()
    } else {
        value.get(name).cloned()
    }
}
fn merged(node: &Node, pin: &str, ty: &schema::Type) -> Result<Value, String> {
    let prefix = format!("{pin}.");
    if node.inputs.keys().any(|p| p.starts_with(&prefix)) {
        let fields = ty.members();
        let values = fields
            .iter()
            .map(|f| merged(node, &format!("{pin}.{}", f.name), &f.value_type))
            .collect::<Result<Vec<_>, _>>()?;
        if matches!(ty, schema::Type::Vector { .. }) {
            Ok(Value::Array(values))
        } else {
            Ok(Value::Object(
                fields
                    .into_iter()
                    .zip(values)
                    .map(|(f, v)| (f.name, v))
                    .collect(),
            ))
        }
    } else {
        match node.inputs.get(pin) {
            Some(Input::Literal { value, value_type }) if value_type == ty => Ok(value.clone()),
            None => Ok(default_value(ty)),
            _ => Err("Disconnect all child pins before recombining.".into()),
        }
    }
}

impl BlueprintEditor {
    pub(super) fn can_connect(&self, a: &Socket, b: &Socket, registry: &Registry) -> bool {
        if a.output == b.output || a.node == b.node {
            return false;
        }
        // Run the same scope, cycle and type checks as committing the connection.
        // The scratch document makes event-island adoption read-only for preview.
        let mut probe = BlueprintEditor {
            asset: self.asset.clone(),
            graph: self.graph,
            playback_timelines: self.playback_timelines.clone(),
            playback_effects: self.playback_effects.clone(),
            ..Default::default()
        };
        probe.connect(a, b, registry).is_ok()
    }
    pub(super) fn promote_reason(
        &self,
        socket: &Socket,
        registry: &Registry,
    ) -> Result<(), String> {
        let SocketType::Value(ty) = &socket.ty else {
            return Err("Only data pins can become variables.".into());
        };
        if !crate::script_values::valid(&default_value(ty), ty) {
            return Err("This type cannot be stored in a Blueprint variable.".into());
        }
        let doc = self.asset.as_ref().ok_or("No Blueprint")?;
        if properties(doc, registry).len() + doc.variables.len() >= 16 {
            return Err("The PSX Blueprint property budget is full (16).".into());
        }
        let node = doc
            .functions
            .iter()
            .flat_map(|g| &g.nodes)
            .find(|n| n.id == socket.node)
            .ok_or("Missing node")?;
        if !socket.output
            && matches!(
                node.inputs.get(&socket.pin),
                Some(Input::Link { .. } | Input::Parameter { .. })
            )
        {
            return Err("Disconnect the input before promoting its default to a variable.".into());
        }
        Ok(())
    }
    pub(super) fn promote_pin(
        &mut self,
        socket: &Socket,
        registry: &Registry,
    ) -> Result<(), String> {
        self.promote_reason(socket, registry)?;
        self.select_node_graph(&socket.node);
        let SocketType::Value(ty) = &socket.ty else {
            unreachable!()
        };
        let doc = self.asset.as_ref().unwrap();
        let graph = &doc.functions[self.graph];
        let source = graph.nodes.iter().find(|n| n.id == socket.node).unwrap();
        let default = if !socket.output {
            match source.inputs.get(&socket.pin) {
                Some(Input::Literal { value, .. }) => value.clone(),
                _ => default_value(ty),
            }
        } else {
            default_value(ty)
        };
        if !crate::script_values::valid(&default, ty) {
            return Err("Fix the pin's default value before promoting it.".into());
        }
        let label = if socket.pin == "value" {
            node_label(source, doc, registry)
        } else {
            socket.label.clone()
        };
        let base = label
            .trim_start_matches("Get ")
            .trim_start_matches("Set ")
            .to_ascii_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect::<String>();
        let base = if crate::scripts::identifier(&base) {
            base
        } else {
            "new_variable".into()
        };
        let mut name = base.clone();
        let inherited = properties(doc, registry);
        let mut suffix = 2;
        while doc.variables.iter().any(|v| v.name == name)
            || inherited.iter().any(|v| v.name == name)
            || functions(doc, registry).iter().any(|f| f.name == name)
            || doc.functions.iter().any(|f| f.name == name)
        {
            name = format!("{base}_{suffix}");
            suffix += 1;
        }
        let at = node_position(doc, graph, &socket.node);
        let has_next = node_sockets_with_assets(
            doc,
            graph,
            source,
            registry,
            &self.playback_timelines,
            &self.playback_effects,
        )
        .iter()
        .any(|pin| pin.output && pin.pin == "next" && pin.ty == SocketType::Exec);
        let variable = id();
        let key = id();
        let doc = self.asset.as_mut().unwrap();
        doc.variables.push(asset::Variable {
            id: variable.clone(),
            name,
            value_type: ty.clone(),
            default,
            editable: !matches!(
                ty,
                schema::Type::SequenceHandle | schema::Type::EffectHandle
            ),
            timeline_animatable: false,
        });
        let graph = &mut doc.functions[self.graph];
        let mut node = Node {
            id: key.clone(),
            kind: if socket.output {
                NodeKind::SetVariable {
                    member: variable.clone(),
                }
            } else {
                NodeKind::GetVariable {
                    member: variable.clone(),
                }
            },
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
        };
        let source = graph
            .nodes
            .iter_mut()
            .find(|n| n.id == socket.node)
            .unwrap();
        if socket.output {
            node.inputs.insert(
                "value".into(),
                Input::Link {
                    node: socket.node.clone(),
                    pin: socket.pin.clone(),
                },
            );
            if has_next {
                let next = source.outputs.entry("next".into()).or_default();
                node.outputs
                    .insert("next".into(), std::mem::replace(next, vec![key.clone()]));
            }
        } else {
            source.inputs.insert(
                socket.pin.clone(),
                Input::Link {
                    node: key.clone(),
                    pin: "value".into(),
                },
            );
        }
        graph.nodes.push(node);
        doc.layout.positions.insert(
            key.clone(),
            [
                at[0] + if socket.output { 260. } else { -260. },
                at[1] + 80.,
            ],
        );
        self.selected = BTreeSet::from([key]);
        self.detail_member = variable;
        self.details = Details::Variable;
        self.wire = None;
        Ok(())
    }
    pub(super) fn split_pin_reason(
        &self,
        socket: &Socket,
        registry: &Registry,
    ) -> Result<(), String> {
        let SocketType::Value(ty) = &socket.ty else {
            return Err("Execution pins cannot be split.".into());
        };
        if ty.members().is_empty() {
            return Err("This type has no splittable fields.".into());
        }
        if socket.pin.split('.').count() >= 16 {
            return Err("Maximum split depth reached.".into());
        }
        let doc = self.asset.as_ref().ok_or("No Blueprint")?;
        let graph = doc
            .functions
            .iter()
            .find(|g| g.nodes.iter().any(|n| n.id == socket.node))
            .ok_or("Missing node")?;
        let node = graph.nodes.iter().find(|n| n.id == socket.node).unwrap();
        if split(doc, node, socket) {
            return Err("Pin is already split.".into());
        }
        if socket.output {
            if doc
                .functions
                .iter()
                .flat_map(|g| g.nodes.iter().map(move |n| (g, n)))
                .any(|(g, n)| {
                    n.inputs
                        .values()
                        .any(|i| input_connection(&g.entry, i) == Some((&socket.node, &socket.pin)))
                })
            {
                return Err("Disconnect the pin before splitting.".into());
            }
        } else {
            if matches!(
                node.inputs.get(&socket.pin),
                Some(Input::Link { .. } | Input::Parameter { .. })
            ) {
                return Err("Disconnect the pin before splitting.".into());
            }
            let params = match &node.kind {
                NodeKind::Call { function } => function_by_id(doc, registry, function)
                    .map(|f| f.parameters.clone())
                    .or_else(|| {
                        doc.functions
                            .iter()
                            .find(|g| g.id == *function)
                            .map(|g| g.parameters.clone())
                    }),
                NodeKind::CallOn { class, function } => {
                    crate::blueprint_ir::call_on_function(registry, class, function)
                        .ok()
                        .map(|f| f.parameters.clone())
                }
                NodeKind::CallParent => Some(graph.parameters.clone()),
                _ => None,
            };
            let root = socket.pin.split('.').next().unwrap();
            if params
                .unwrap_or_default()
                .iter()
                .any(|p| p.name == root && p.direction == schema::Direction::MutableReference)
            {
                return Err(
                    "Mutable reference inputs require a whole value with writable storage.".into(),
                );
            }
        }
        Ok(())
    }
    pub(super) fn split_pin(&mut self, socket: &Socket, registry: &Registry) -> Result<(), String> {
        self.split_pin_reason(socket, registry)?;
        let SocketType::Value(ty) = &socket.ty else {
            unreachable!()
        };
        let doc = self.asset.as_mut().unwrap();
        if !socket.output {
            let node = doc
                .functions
                .iter_mut()
                .flat_map(|g| &mut g.nodes)
                .find(|n| n.id == socket.node)
                .unwrap();
            let value = match node.inputs.get(&socket.pin) {
                Some(Input::Literal { value, .. }) => value.clone(),
                _ => default_value(ty),
            };
            if !crate::script_values::valid(&value, ty) {
                return Err("Fix the pin's invalid default value before splitting.".into());
            }
            for (index, field) in ty.members().into_iter().enumerate() {
                node.inputs.insert(
                    format!("{}.{}", socket.pin, field.name),
                    Input::Literal {
                        value: child_value(&value, ty, index, &field.name)
                            .unwrap_or_else(|| default_value(&field.value_type)),
                        value_type: field.value_type,
                    },
                );
            }
            node.inputs.insert(
                socket.pin.clone(),
                Input::Literal {
                    value_type: ty.clone(),
                    value,
                },
            );
        }
        doc.layout.split_pins.insert(key(socket));
        self.wire = None;
        Ok(())
    }
    pub(super) fn recombine_pin(&mut self, socket: &Socket, apply: bool) -> Result<(), String> {
        let (parent, _) = socket
            .pin
            .rsplit_once('.')
            .ok_or("This pin is not split.")?;
        let prefix = format!("{parent}.");
        let doc = self.asset.as_mut().ok_or("No Blueprint")?;
        if socket.output
            && doc.functions.iter().any(|g| {
                g.nodes.iter().any(|n| {
                    n.inputs.values().any(|i| {
                        input_connection(&g.entry, i).is_some_and(|(node, pin)| {
                            node == socket.node && pin.starts_with(&prefix)
                        })
                    })
                })
            })
        {
            return Err("Disconnect all child pins before recombining.".into());
        }
        let node = doc
            .functions
            .iter_mut()
            .flat_map(|g| &mut g.nodes)
            .find(|n| n.id == socket.node)
            .ok_or("Missing node")?;
        if !socket.output {
            if node.inputs.iter().any(|(pin, input)| {
                pin.starts_with(&prefix)
                    && matches!(input, Input::Link { .. } | Input::Parameter { .. })
            }) {
                return Err("Disconnect all child pins before recombining.".into());
            }
            let Some(Input::Literal { value_type, .. }) = node.inputs.get(parent) else {
                return Err("Missing parent pin.".into());
            };
            let ty = value_type.clone();
            let value = merged(node, parent, &ty)?;
            if !crate::script_values::valid(&value, &ty) {
                return Err("Fix child values before recombining.".into());
            }
            if apply {
                node.inputs.retain(|pin, _| !pin.starts_with(&prefix));
                node.inputs.insert(
                    parent.into(),
                    Input::Literal {
                        value_type: ty,
                        value,
                    },
                );
            }
        }
        if apply {
            let parent_key = key(&Socket {
                pin: parent.into(),
                ..socket.clone()
            });
            doc.layout
                .split_pins
                .retain(|k| k != &parent_key && !k.starts_with(&format!("{parent_key}.")));
            self.wire = None;
        }
        Ok(())
    }
    pub(super) fn pin_popup(&mut self, ui: &Ui, registry: &Registry) {
        if let Some(_popup) = ui.begin_popup("Blueprint Pin")
            && let Some(socket) = self.pin_menu.clone()
        {
            ui.text_disabled("Pin Actions");
            let promote = self.promote_reason(&socket, registry);
            if ui
                .menu_item_config("Promote to Variable")
                .enabled(promote.is_ok())
                .build()
                && let Err(error) = self.promote_pin(&socket, registry)
            {
                self.error = Some(error);
            }
            record_control(ui, "Promote to Variable");
            if ui.is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED)
                && let Err(reason) = promote
            {
                ui.tooltip_text(reason);
            }
            let split = self.split_pin_reason(&socket, registry);
            if ui
                .menu_item_config("Split Struct Pin")
                .enabled(split.is_ok())
                .build()
                && let Err(error) = self.split_pin(&socket, registry)
            {
                self.error = Some(error);
            }
            record_control(ui, "Split Struct Pin");
            if ui.is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED)
                && let Err(reason) = split
            {
                ui.tooltip_text(reason);
            }
            let recombine = self.recombine_pin(&socket, false);
            if ui
                .menu_item_config("Recombine Struct Pin")
                .enabled(recombine.is_ok())
                .build()
                && let Err(error) = self.recombine_pin(&socket, true)
            {
                self.error = Some(error);
            }
            record_control(ui, "Recombine Struct Pin");
            if ui.is_item_hovered_with_flags(imgui::ItemHoveredFlags::ALLOW_WHEN_DISABLED)
                && let Err(reason) = recombine
            {
                ui.tooltip_text(reason);
            }
        }
    }
}
