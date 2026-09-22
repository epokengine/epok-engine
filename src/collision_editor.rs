pub fn inspector(ui: &imgui::Ui, entity: &mut crate::scene::Actor) {
    if entity.collider.is_none() {
        return;
    }
    let mut remove = false;
    let open = crate::gui::section(ui, "Box Collider", || {
        remove = ui.menu_item("Remove Box Collider");
    });
    if remove {
        entity.collider = None;
        ui.separator();
        return;
    }
    if !open {
        return;
    }
    let collider = entity.collider.as_mut().unwrap();
    crate::gui::toggle(ui, "Enabled##collider", &mut collider.enabled);
    crate::gui::toggle(ui, "Trigger##collider", &mut collider.trigger);
    crate::gui::Drag::new(crate::gui::field(ui, "Center##collider"))
        .speed(0.01)
        .range(-128., 128.)
        .build_array(ui, &mut collider.center);
    crate::gui::Drag::new(crate::gui::field(ui, "Half Extents##collider"))
        .speed(0.01)
        .range(1. / 4096., 128.)
        .build_array(ui, &mut collider.half_extents);
    let mut layer = format!("{:08X}", collider.layer);
    if ui
        .input_text(crate::gui::field(ui, "Layer Bits (hex)"), &mut layer)
        .chars_hexadecimal(true)
        .build()
        && let Ok(value) = u32::from_str_radix(&layer, 16)
    {
        collider.layer = value.max(1);
    }
    let mut mask = format!("{:08X}", collider.mask);
    if ui
        .input_text(crate::gui::field(ui, "Mask Bits (hex)"), &mut mask)
        .chars_hexadecimal(true)
        .build()
        && let Ok(value) = u32::from_str_radix(&mask, 16)
    {
        collider.mask = value;
    }
    ui.text_wrapped("Boxes use conservative world AABBs after hierarchy transforms. Triggers report enter/stay/exit and do not block movement. Layer/mask values are bit fields.");
    ui.separator();
}
