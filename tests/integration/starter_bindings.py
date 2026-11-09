"""Validate the ordinary class references recorded by a template installation."""
from verify_camera_rigs import documents


def player_roles(project, scene, template, flavor):
    path = project / "ProjectSettings/Template.epoksettings"
    if not path.exists():
        return None  # Older frozen acceptance projects have no installation file.
    installed = documents.loads(path.read_text(encoding="utf-8"))
    assert installed["template"] == template and installed["gameplay"] == flavor, installed
    roles = installed["roles"]
    player = next(a for a in scene["actors"] if a["class"].get("class_id") == roles["player"]["class_id"])
    camera = next(a for a in scene["actors"] if a["class"].get("class_id") == roles["camera"]["class_id"])
    for actor, role in ((player, "controller"), (player, "locomotion"), (camera, "camera_input")):
        assert any(c["class"].get("class_id") == roles[role]["class_id"] for c in actor["components"]), role
    assert player["properties"]["camera"] == camera["id"]
    assert camera["properties"]["target"] == player["id"]
    assert camera.get("attach") is None
    assert any(c["id"] == player["properties"]["visual"] for a in scene["actors"] for c in a["components"])
    return player, camera


def platformer_roles(project, scene, flavor):
    path = project/"ProjectSettings/Template.epoksettings"
    if not path.exists():
        return None  # Frozen pre-migration projects remain replayable.
    installed = documents.loads(path.read_text(encoding="utf-8"))
    assert installed["template"] == "platformer-2d" and installed["gameplay"] == flavor
    roles = installed["roles"]
    player = next(a for a in scene["actors"] if a["class"].get("class_id") == roles["player"]["class_id"])
    camera = next(a for a in scene["actors"] if any(c["class"].get("class_id") == roles["camera"]["class_id"] for c in a["components"]))
    assert len(player["components"]) == 7 and camera["attach"]["actor"] == player["id"]
    assert any(c["id"] == player["properties"]["camera"] for c in camera["components"])
    assert any(c["id"] == player["properties"]["visual"] for c in player["components"])
    for role in ("controller", "locomotion"):
        assert any(c["class"].get("class_id") == roles[role]["class_id"] for c in player["components"])
    coins = [a for a in scene["actors"] if any(c["class"].get("class_id") == roles["pickup"]["class_id"] for c in a["components"])]
    assert len(coins) == 8
    for actor in coins:
        pickup = next(c for c in actor["components"] if c["class"].get("class_id") == roles["pickup"]["class_id"])
        assert pickup["properties"]["accepted_initiator"] == player["id"] and pickup["properties"]["auto_collect"] is True
    return player, camera, coins
