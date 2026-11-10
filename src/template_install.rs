//! Template installation choices and semantic roles reference the existing class
//! identities. They do not define classes or change the project's authoring mode.
use crate::{
    actor_document::ClassReference,
    workspace::{GameplayFlavor, Template},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

pub const PATH: &str = "ProjectSettings/Template.epoksettings";
const VERSION: u32 = 1;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Installation {
    pub version: u32,
    pub template: String,
    pub gameplay: String,
    pub roles: BTreeMap<String, ClassReference>,
}
impl Installation {
    fn validate(&self) -> Result<(), String> {
        if self.version != VERSION
            || Template::parse(&self.template).is_none()
            || GameplayFlavor::parse(&self.gameplay).is_none()
        {
            return Err("Unsupported template installation version, template or gameplay flavor. The file is preserved.".into());
        }
        for (role, class) in &self.roles {
            if role.is_empty()
                || !role
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
                || class.name.is_empty()
                || class
                    .class_id
                    .as_ref()
                    .and_then(|id| uuid::Uuid::parse_str(id).ok())
                    .is_none_or(|id| id.is_nil())
            {
                return Err(format!(
                    "Template role {role:?} requires a class name and stable class UUID."
                ));
            }
        }
        Ok(())
    }
}

pub fn read(root: &Path) -> Result<Option<Installation>, String> {
    let path = root.join(PATH);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let installation: Installation = crate::document::from_slice(&bytes)?;
    installation.validate()?;
    Ok(Some(installation))
}

pub fn write<'a>(
    root: &Path,
    template: Template,
    gameplay: GameplayFlavor,
    roles: impl IntoIterator<Item = (&'a str, ClassReference)>,
) -> Result<(), String> {
    let installation = Installation {
        version: VERSION,
        template: template.key().into(),
        gameplay: gameplay.key().into(),
        roles: roles
            .into_iter()
            .map(|(role, class)| (role.into(), class))
            .collect(),
    };
    installation.validate()?;
    let mut transaction = crate::helper_install::Transaction::new(root);
    transaction.settings(PATH, &crate::document::to_vec(&installation)?)?;
    transaction.commit();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_installation_retains_identity_and_rejects_invalid_edits_without_rewriting() {
        let root = crate::workspace::tests::temp("template-role-file");
        std::fs::create_dir_all(root.join("ProjectSettings")).unwrap();
        assert!(read(&root).unwrap().is_none());
        let reference = ClassReference::new("OriginalName", crate::object_model::ACTOR3D_ID);
        write(
            &root,
            Template::Basic,
            GameplayFlavor::Lua,
            [("game_root", reference)],
        )
        .unwrap();
        let installation = read(&root).unwrap().unwrap();
        assert_eq!(installation.gameplay, "lua");
        let native = crate::blueprint::native_registry(&root, &[]).unwrap();
        let model = native.model().unwrap();
        assert_eq!(
            installation.roles["game_root"]
                .resolve(&model)
                .unwrap()
                .cpp_name,
            "epok::Actor3D"
        );
        for (key, value) in [
            ("version", serde_json::json!(2)),
            ("gameplay", serde_json::json!("unknown")),
            (
                "roles",
                serde_json::json!({"bad role":{"name":"Actor","class_id":crate::object_model::ACTOR3D_ID}}),
            ),
        ] {
            let mut malformed = serde_json::to_value(&installation).unwrap();
            malformed[key] = value;
            let bytes = crate::document::to_vec(&malformed).unwrap();
            std::fs::write(root.join(PATH), &bytes).unwrap();
            assert!(read(&root).is_err());
            assert_eq!(std::fs::read(root.join(PATH)).unwrap(), bytes);
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
