//! Usability scenarios for the 2D editor: common tasks done in a new Platformer 2D project
//! through the real editor panels (the 2D view's toolbar, the Hierarchy and the
//! Inspector) with simulated mouse and keyboard input. Each scenario counts the
//! interactions it takes — a click, a typed word or a key press is one — and checks the
//! result works without a script and leaves no warning behind. Placing nodes in the view
//! is not counted. `cargo test world2d_usability -- --nocapture` prints the tally.
use crate::{
    editor::Editor, physics2d, scene_view_mode::SceneViewMode, viewport2d, world2d,
    world2d_warnings,
};
use imgui::{Condition, Key, MouseButton};
use uuid::Uuid;

struct Session {
    context: crate::gui::tests::ImguiContext,
    editor: Editor,
    steps: usize,
}

impl Session {
    fn new(name: &str) -> Self {
        let mut context = crate::gui::tests::imgui_context();
        crate::gui::configure_input(context.io_mut());
        context.io_mut().display_size = [1600., 900.];
        context.io_mut().delta_time = 1. / 60.;
        context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        context.fonts().build_rgba32_texture();
        context.load_ini_settings(
            "[Window][###Hierarchy]\nPos=900,0\nSize=330,900\nCollapsed=0\n\
             [Window][###Inspector]\nPos=1230,0\nSize=370,900\nCollapsed=0\n",
        );
        let root = crate::workspace::tests::temp(name);
        let project =
            crate::workspace::create(&root, "Usability", crate::workspace::Template::Platformer2D)
                .unwrap();
        let mut editor = Editor::open(project).unwrap();
        editor.auto_build = false;
        editor.set_scene_view_mode(SceneViewMode::TwoD);
        editor.select_actor(None);
        let mut session = Self {
            context,
            editor,
            steps: 0,
        };
        for _ in 0..3 {
            session.frame();
        }
        session
    }
    fn frame(&mut self) {
        crate::gui::clear_script_controls();
        let ui = self.context.frame();
        let e = &mut self.editor;
        ui.window("Scene")
            .position([0., 0.], Condition::Always)
            .size([900., 900.], Condition::Always)
            .build(|| crate::world2d_editor::viewport(ui, e));
        crate::gui::hierarchy(ui, e);
        crate::gui::inspector(ui, e);
        self.context.render();
    }
    fn control(&mut self, label: &str) -> [f32; 2] {
        for _ in 0..8 {
            self.frame();
            if let Some(point) = crate::gui::script_control(label) {
                return point;
            }
        }
        panic!(
            "no control {label:?} on screen; drawn: {:?}",
            crate::gui::script_control_labels()
        );
    }
    fn shown(&mut self, label: &str) -> bool {
        self.frame();
        self.frame();
        crate::gui::script_control(label).is_some()
    }
    fn click(&mut self, label: &str) {
        let point = self.control(label);
        self.context.io_mut().add_mouse_pos_event(point);
        self.frame();
        self.context
            .io_mut()
            .add_mouse_button_event(MouseButton::Left, true);
        self.frame();
        self.context
            .io_mut()
            .add_mouse_button_event(MouseButton::Left, false);
        self.frame();
        self.frame();
        self.steps += 1;
    }
    fn type_text(&mut self, text: &str) {
        for c in text.chars() {
            self.context.io_mut().add_input_character(c);
        }
        self.frame();
        self.frame();
        self.steps += 1;
    }
    fn press(&mut self, key: Key) {
        self.context.io_mut().add_key_event(key, true);
        self.frame();
        self.context.io_mut().add_key_event(key, false);
        self.frame();
        self.frame();
        self.steps += 1;
    }
    fn selected(&self) -> Uuid {
        self.editor.selected_actor.expect("a node is selected")
    }
    fn actor(&self, id: Uuid) -> &crate::scene::Actor {
        let index = self.editor.scene.actor_index(id).unwrap();
        &self.editor.scene.actors[index]
    }
    fn named(&self, name: &str) -> Uuid {
        self.editor
            .scene
            .actors
            .iter()
            .find(|a| a.name == name)
            .unwrap_or_else(|| panic!("no node {name}"))
            .id
    }
    fn record<T: serde::de::DeserializeOwned + Default>(&self, id: Uuid, class: &str) -> T {
        world2d::read(world2d::component(self.actor(id), class).expect("the component"))
    }
    fn warnings(&self, id: Uuid) -> Vec<world2d_warnings::Warning> {
        world2d_warnings::of(
            &self.editor.scene,
            self.editor.scene.actor_index(id).unwrap(),
        )
    }
    /// Adds a node from the toolbar's Add 2D Node menu: a click, a search, Enter.
    fn add_node(&mut self, search: &str) -> Uuid {
        let before = self.editor.scene.actors.len();
        self.click("Add 2D Node");
        self.type_text(search);
        self.press(Key::Enter);
        assert_eq!(
            self.editor.scene.actors.len(),
            before + 1,
            "{search} made a node"
        );
        self.selected()
    }
    fn done(self, task: &str, budget: usize) {
        println!("{task}: {} interactions, no script", self.steps);
        assert!(
            self.steps <= budget,
            "{task} took {} interactions",
            self.steps
        );
        assert!(
            self.editor.last_error.is_none(),
            "{:?}",
            self.editor.last_error
        );
    }
}

#[test]
fn world2d_usability_a_crate_the_player_can_push() {
    let mut s = Session::new("usability-crate");
    let crate_node = s.add_node("rigid");
    let texture = s.record::<world2d::Sprite2D>(s.named("PushCrate"), world2d::SPRITE2D);
    let source = s
        .editor
        .assets
        .index
        .resolve(texture.texture.unwrap())
        .unwrap()
        .meta
        .source
        .clone();
    let stem = std::path::Path::new(&source)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_lowercase();
    s.click("Sprite texture");
    s.type_text(&stem);
    s.press(Key::Enter);
    let sprite: world2d::Sprite2D = s.record(crate_node, world2d::SPRITE2D);
    assert_eq!(sprite.texture, texture.texture);
    let body: physics2d::RigidBody2D = s.record(crate_node, world2d::RIGID_BODY2D);
    assert!(body.pushable, "characters push it without a script");
    assert!(world2d::component(s.actor(crate_node), world2d::COLLIDER2D).is_some());
    assert!(s.warnings(crate_node).is_empty());
    s.done("A crate the player can push", 6);
}

#[test]
fn world2d_usability_a_lamp_on_a_rope_fixed_from_its_warning() {
    let mut s = Session::new("usability-rope");
    let lamp = s.add_node("rigid");
    let joint = s.add_node("rope");
    // The joint has nothing to hold yet: the Hierarchy and the Inspector both say so.
    assert!(s.warnings(joint)[0].text.contains("no Body B"));
    assert!(s.shown(&format!("Hierarchy warning {joint}")));
    s.click("Kind: Rope");
    s.click("Body B");
    s.click("Body B: RigidBody2D");
    let record: physics2d::Joint2D = s.record(joint, world2d::JOINT2D);
    assert_eq!(record.kind, physics2d::JointKind::Rope);
    assert_eq!(record.body_b, Some(lamp));
    assert!(s.warnings(joint).is_empty() && s.warnings(lamp).is_empty());
    assert!(!s.shown(&format!("Hierarchy warning {joint}")));
    s.done("A lamp hanging on a rope", 9);
}

#[test]
fn world2d_usability_an_enemy_that_chases_the_player() {
    let mut s = Session::new("usability-chase");
    let enemy = s.add_node("agent");
    s.click("Target");
    s.type_text("player");
    s.press(Key::Enter);
    let agent: crate::navigation2d::NavigationAgent2D =
        s.record(enemy, world2d::NAVIGATION_AGENT2D);
    assert_eq!(agent.target, Some(s.named("Player")));
    assert!(
        agent.drives_body,
        "the agent walks its node without a script"
    );
    let (path, _) = crate::world2d_navigation_editor::preview(&s.editor, enemy).unwrap();
    assert!(!path.is_empty(), "the view previews the path it will take");
    assert!(s.warnings(enemy).is_empty());
    s.done("An enemy that chases the player", 6);
}

#[test]
fn world2d_usability_a_picture_in_picture_on_the_hud() {
    let mut s = Session::new("usability-picture");
    let picture = s.add_node("minimap");
    assert_eq!(
        s.warnings(picture)[0].fix,
        Some(world2d_warnings::Fix::ShowPicture)
    );
    s.click("Show it on screen");
    let screen = s
        .editor
        .scene
        .actors
        .iter()
        .find(|a| {
            world2d::component(a, world2d::SPRITE2D)
                .is_some_and(|c| world2d::read::<world2d::Sprite2D>(c).viewport == Some(picture))
        })
        .expect("a sprite shows the picture");
    let canvas = s.actor(screen.attach.as_ref().expect("on a canvas").actor);
    let layer: world2d::CanvasLayer2D =
        world2d::read(world2d::component(canvas, world2d::CANVAS_LAYER2D).expect("a canvas layer"));
    assert!(!layer.follow_viewport, "drawn in screen pixels, like a HUD");
    let size = world2d::read::<viewport2d::SubViewport2D>(
        world2d::component(s.actor(picture), world2d::SUBVIEWPORT2D).unwrap(),
    )
    .picture();
    let root = screen.components.iter().find(|c| c.root).unwrap();
    assert_eq!(
        root.properties["position"][0].as_f64(),
        Some(f64::from(320 - size[0] / 2 - 8)),
        "in the top-right corner"
    );
    assert_eq!(s.selected(), picture, "the fixed node stays selected");
    assert!(s.warnings(picture).is_empty());
    s.done("A picture-in-picture on the HUD", 4);
}

#[test]
fn world2d_usability_physics_on_an_existing_sprite() {
    let mut s = Session::new("usability-falling");
    let cloud = s.named("Cloud0");
    s.click("Hierarchy search");
    s.type_text("Cloud0");
    s.click(&format!("Hierarchy actor {cloud}"));
    assert_eq!(s.selected(), cloud);
    s.click("Add Component");
    s.type_text("rigid");
    s.press(Key::Enter);
    assert!(world2d::component(s.actor(cloud), world2d::RIGID_BODY2D).is_some());
    assert_eq!(
        s.warnings(cloud)[0].fix,
        Some(world2d_warnings::Fix::AddCollider)
    );
    s.click("Add a Collider2D");
    let collider: world2d::Collider2D = s.record(cloud, world2d::COLLIDER2D);
    let texture = s
        .record::<world2d::Sprite2D>(cloud, world2d::SPRITE2D)
        .texture
        .unwrap();
    let data = &s.editor.scene.textures[&texture];
    assert_eq!(
        collider.size,
        [f32::from(data.width), f32::from(data.height)],
        "fitted to the sprite"
    );
    assert!(s.warnings(cloud).is_empty());
    s.done("Physics on an existing sprite", 7);
}

#[test]
fn world2d_usability_a_split_screen_for_two_players() {
    let mut s = Session::new("usability-split");
    let second = s.add_node("camera");
    let split = s.add_node("split");
    assert!(
        s.warnings(split)[0]
            .text
            .contains("two nodes with a Camera2D")
    );
    s.click("Left camera");
    s.click("Left camera: Player");
    s.click("Right camera");
    s.click("Right camera: Camera2D");
    let record: viewport2d::SplitScreen2D = s.record(split, world2d::SPLIT_SCREEN2D);
    assert_eq!(record.cameras, [Some(s.named("Player")), Some(second)]);
    assert!(s.warnings(split).is_empty());
    s.done("A split screen for two players", 10);
}
