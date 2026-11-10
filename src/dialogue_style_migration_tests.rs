use super::*;

fn legacy() -> serde_json::Value {
    serde_json::json!({"version":1,"presentation":"portrait_conversation","font":Uuid::new_v4(),"panel":Uuid::new_v4(),"panel_borders":[4,4,4,4],"text_color":[0.9,0.8,0.7],"panel_color":[0.1,0.2,0.3],"margins":[7,8,9,10],"portrait_size":[48,64],"continuation_indicator":true,"line_policy":{"milliseconds_per_glyph":17,"auto_advance_ms":null},"open_sound":null,"advance_sound":null,"close_sound":null})
}
#[test]
fn legacy_style_migrates_only_on_save_and_preserves_the_original_package() {
    let root = crate::workspace::tests::temp("style-ownership-migration");
    crate::workspace::create(&root, "Style Migration", crate::workspace::Template::Basic).unwrap();
    let source = legacy();
    let id = create_package(
        &root,
        "assets/Old.epokasset",
        Kind::DialogueStyle,
        crate::document::to_vec(&source).unwrap(),
    )
    .unwrap();
    let index = assets::scan(&root, &mut Default::default());
    let record = index.resolve(id).unwrap();
    let original = assets::read_bounded(&record.path).unwrap();
    let mut style = style_document(record).unwrap();
    assert_eq!(style.version, STYLE_VERSION);
    assert_eq!(style.line_policy.milliseconds_per_glyph, 17);
    style.validate_resources(&index).unwrap();
    assert_eq!(
        assets::read_bounded(&record.path).unwrap(),
        original,
        "read must not rewrite a v1 document"
    );
    let encoded = crate::document::to_vec(&style).unwrap();
    let current: serde_json::Value = crate::document::from_slice(&encoded).unwrap();
    assert!(current.get("font").is_none() && current.get("presentation").is_none());
    let mut mislabeled = source.clone();
    mislabeled["version"] = serde_json::json!(2);
    assert!(DialogueStyle::parse(&crate::document::to_vec(&mislabeled).unwrap()).is_err());
    style.line_policy.milliseconds_per_glyph = 8;
    save_style(&root, record, &style, &index).unwrap();
    let backup = root
        .join(".epok/migrations/dialogue-style")
        .join(id.to_string())
        .join(format!("{}.epokasset", assets::hash(&original)));
    assert_eq!(assets::read_bounded(&backup).unwrap(), original);
    assert_eq!(style_document(record).unwrap(), style);
    assert!(
        save_style(&root, record, &style, &index).is_err(),
        "stale save must not overwrite the conversion"
    );
    let fresh = assets::scan(&root, &mut Default::default());
    assert_eq!(
        fresh.assets[&id].len(),
        1,
        "migration backups must not enter the asset catalog"
    );
    let current = fresh.resolve(id).unwrap();
    let bytes = assets::read_bounded(&current.path).unwrap();
    save_style(&root, current, &style, &fresh).unwrap();
    assert_eq!(assets::read_bounded(&current.path).unwrap(), bytes);
    let mut invalid = legacy();
    invalid["version"] = serde_json::json!(99);
    assert!(DialogueStyle::parse(&crate::document::to_vec(&invalid).unwrap()).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
