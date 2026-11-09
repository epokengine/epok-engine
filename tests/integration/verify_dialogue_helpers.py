"""Execute shared dialogue helpers through all providers in both spatial domains."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import socket
import struct
import subprocess
import sys
import time
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents

EDITOR = ROOT / "target/debug/epok-editor.exe"
OUT = ROOT / "artifacts/gameplay-helpers/p3/providers"
PROJECT = OUT / "project"
WIDGET_PROVIDER = "native"
NS = uuid.UUID("18263f9d-d972-530a-91ac-739c4ed39dc4")
MAGIC = 0x444c4833
WORDS = 24


def identity(name):
    return str(uuid.uuid5(NS, "fixture.dialogue-helpers." + name))


def compact(name):
    return int.from_bytes(hashlib.sha256(uuid.UUID(identity(name)).bytes).digest()[:4], "little")


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    documents.write_text(path, documents.dumps(value) if isinstance(value, dict) else value)


def run(*args, log=None):
    result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True,
                            text=True, encoding="utf-8", timeout=900)
    if log:
        (OUT / log).write_text(result.stdout + result.stderr, encoding="utf-8")
    assert result.returncode == 0, result.stdout + result.stderr
    return result.stdout


def package(path, kind, name, value):
    source = (json.dumps(value, ensure_ascii=False) if kind == "StringTable" else documents.dumps(value)).encode()
    meta = {"version": 2, "id": identity(name), "kind": kind, "importer_version": 1,
            "source": path, "source_hash": hashlib.sha256(source).hexdigest(), "settings": {"type": "Authored"}}
    encoded = json.dumps(meta).encode()
    destination = PROJECT / path
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(b"EPOKAS01" + struct.pack("<II", len(encoded), len(source)) + encoded + source)


def prepare():
    OUT.mkdir(parents=True, exist_ok=True)
    if not PROJECT.exists():
        run("--create-project", PROJECT, "--template", "basic", "--name", "Dialogue Helper Providers")
    write(PROJECT / "assets/scripts/ProjectionProbe.hpp", (ROOT / "tests/fixtures/native-scenes/ProjectionProbe.hpp").read_text())
    write(PROJECT / "assets/scripts/AnchorProbe.hpp", (ROOT / "tests/fixtures/native-scenes/AnchorProbe.hpp").read_text())
    header = r'''#pragma once
#include "epok.hpp"
extern "C" {inline uint32_t dialogue_helper_probe[144]={};inline uint32_t dialogue_helper_turn=0;inline uint32_t dialogue_widget_probe[10]={};inline uint32_t dialogue_focus_probe[4]={};inline uint32_t dialogue_anchor_vm_probe[3]={};}
class EPOK_CLASS(Blueprintable,Id="WIDGET_BASE") DialogueWidgetProbe:public epok::DialoguePresenter {
public:
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::ScreenAnchorComponent") epok::ObjectId anchor_probe{};
 EPOK_FUNCTION(BlueprintCallable) void observe_anchor(epok::ScreenAnchorStatus refreshed,epok::ScreenAnchorStatus state,bool valid){++dialogue_anchor_vm_probe[0];dialogue_anchor_vm_probe[1]+=refreshed==epok::ScreenAnchorStatus::Idle;dialogue_anchor_vm_probe[2]+=state==epok::ScreenAnchorStatus::Idle&&valid;}
 static constexpr uint64_t static_class_id=epok::detail::compact_class_id("WIDGET_BASE");
 uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
 EPOK_FUNCTION(BlueprintCallable) void observe_widget(epok::DialogueSnapshot state,uint32_t operation){
  auto* p=dialogue_widget_probe;const auto event=operation%100;p[8]=operation/100;
  auto* line=epok::interaction::actor(body);auto* question=epok::interaction::actor(prompt);auto* next=epok::interaction::actor(continuation);
  if(event==1){++p[0];p[3]+=state.phase==epok::DialoguePhase::Executing&&state.generation&&state.source.valid()&&state.initiator.valid()&&state.presenter==id();}
  if(event==2){++p[1];p[4]+=(state.phase==epok::DialoguePhase::Completed||state.phase==epok::DialoguePhase::Cancelled||state.phase==epok::DialoguePhase::Failed)&&state.generation&&state.source.valid()&&state.initiator.valid()&&state.presenter==id();p[9]+=line&&question&&next&&!line->data()->text.enabled&&!question->data()->text.enabled&&!next->data()->text.enabled&&!epok::GameplayInputLibrary::valid(epok::GameplayInputLibrary::owner_capture(state.source,epok::GameplayInputLibrary::all_actions(),state.port));}
  if(event==3){++p[2];p[5]|=question&&question->data()->text.enabled&&state.choices>0;p[6]|=next&&next->data()->text.enabled&&state.phase==epok::DialoguePhase::Waiting;p[7]|=!line||line->data()->rect.position[0].raw()!=24*4096||line->data()->rect.position[1].raw()!=104*4096||line->data()->rect.size[0].raw()!=272*4096||line->data()->rect.size[1].raw()!=32*4096;
   if(state.phase==epok::DialoguePhase::Selecting){
    auto* owner=epok::interaction::actor(choice0);
    if(owner)for(size_t i=0;i<owner->component_count();++i)if(auto* focus=epok::active_object_registry->resolve<epok::FocusableComponent>(owner->component_id(i))){dialogue_focus_probe[0]|=focus->outline_enabled();focus->set_outline(false);const bool off=!focus->outline_enabled();focus->set_outline(true);dialogue_focus_probe[3]|=off&&focus->outline_enabled();}
    dialogue_focus_probe[1]|=epok::hud_stats.rectangles>=5&&!epok::hud_stats.dropped;
   }
  }
 }
};
class EPOK_CLASS(Blueprintable,Id="CPP_WIDGET") CppDialogueWidget:public DialogueWidgetProbe {
public:
 static constexpr uint64_t static_class_id=epok::detail::compact_class_id("CPP_WIDGET");
 uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
 void dialogue_opened(epok::DialogueSnapshot state) override{DialogueWidgetProbe::dialogue_opened(state);observe_widget(state,101);}
 void dialogue_closed(epok::DialogueSnapshot state) override{DialogueWidgetProbe::dialogue_closed(state);observe_widget(state,102);}
 void present(epok::DialogueSnapshot state) override{DialogueWidgetProbe::present(state);observe_widget(state,103);}
};
class EPOK_CLASS(Blueprintable,Owners=World3D|World2D,Id="BASE") DialogueHelperProbe:public epok::DialogueInteractionComponent {
public:
 EPOK_PROPERTY(EditAnywhere) uint32_t probe_slot=0;
 EPOK_PROPERTY(EditAnywhere) bool allowed=true;
 uint32_t* output() const{return dialogue_helper_probe+probe_slot*24;}
 EPOK_FUNCTION(BlueprintEvent,BlueprintPure,Id="CAN_INTERACT") bool can_interact(epok::ObjectId initiator) const override{return allowed&&epok::DialogueInteractionComponent::can_interact(initiator);}
 EPOK_FUNCTION(BlueprintEvent,BlueprintPure,Id="CONDITION") virtual bool condition(epok::ObjectId initiator) const{return allowed&&initiator.valid();}
 EPOK_FUNCTION(BlueprintEvent,Id="ACTION") virtual void award(epok::ObjectId initiator,uint32_t port){if(initiator.valid()&&port==0){auto* p=output();p[7]=p[7]+1;auto* map=epok::active_object_registry->resolve<epok::DialogueRoleBindingsComponent>(bindings);p[19]=map&&map->receiver(0)==id();}}
 EPOK_FUNCTION(BlueprintEvent,Id="STARTED") void dialogue_started(epok::ObjectId initiator,uint32_t port,epok::DialogueHandle session) override{epok::DialogueInteractionComponent::dialogue_started(initiator,port,session);auto* p=output();p[4]=p[4]+1;}
 void dialogue_node_entered(uint32_t node) override{epok::DialogueInteractionComponent::dialogue_node_entered(node);auto* p=output();p[6]=p[6]+1;}
 void dialogue_finished(epok::ObjectId initiator,uint32_t port,epok::DialogueResult result) override{epok::DialogueInteractionComponent::dialogue_finished(initiator,port,result);auto* p=output();p[8]=p[8]+1;p[16]=uint32_t(result);}
 EPOK_FUNCTION(BlueprintCallable) void observe(epok::DialogueSnapshot state,epok::DialogueHandle handle){if(state.source==id()&&state.initiator.valid()&&state.generation==handle.generation&&epok::DialogueSessionLibrary::valid(handle)){auto* p=output();p[5]=p[5]+1;}}
 EPOK_FUNCTION(BlueprintCallable) void observe_start(epok::DialogueStartResult started){if(started.result==epok::DialogueResult::Accepted&&epok::DialogueSessionLibrary::valid(started.session)){auto* p=output();p[18]=p[18]+1;}}
 EPOK_FUNCTION(BlueprintEvent,Id="BEGIN") virtual epok::DialogueStartResult begin_dialogue(epok::ObjectId initiator){auto value=epok::DialogueSessionLibrary::start(id(),initiator,0);observe_start(value);return value;}
 EPOK_FUNCTION(BlueprintEvent) virtual epok::DialogueResult advance_session(epok::DialogueHandle handle){return epok::DialogueSessionLibrary::advance(handle);}
 EPOK_FUNCTION(BlueprintEvent) virtual epok::DialogueResult cancel_session(epok::DialogueHandle handle){return epok::DialogueSessionLibrary::cancel(handle);}
 EPOK_FUNCTION(BlueprintEvent) virtual epok::DialogueResult select_page(epok::DialogueHandle handle){return epok::DialogueSessionLibrary::move_selection(handle,4);}
};
class EPOK_CLASS(Blueprintable,Owners=World3D|World2D,Id="CPP") CppDialogueHelperProbe:public DialogueHelperProbe {
public:
 bool can_interact(epok::ObjectId initiator) const override{return DialogueHelperProbe::can_interact(initiator);}
 bool condition(epok::ObjectId initiator) const override{return DialogueHelperProbe::condition(initiator);}
 void award(epok::ObjectId initiator,uint32_t port) override{DialogueHelperProbe::award(initiator,port);}
 void dialogue_started(epok::ObjectId initiator,uint32_t port,epok::DialogueHandle handle) override{DialogueHelperProbe::dialogue_started(initiator,port,handle);observe(epok::DialogueSessionLibrary::snapshot(handle),handle);}
};
class EPOK_CLASS(Blueprintable,Owners=World3D|World2D,Id="DRIVER") DialogueHelperDriver:public epok::ActorComponent {
 unsigned phase=0,selecting_frames=0;epok::DialogueHandle handle{},old{};
public:
 EPOK_PROPERTY(EditAnywhere,ComponentClass="DialogueHelperProbe") epok::ObjectId target{};
 void on_frame(uint32_t) override{
  auto* value=epok::active_object_registry->resolve<DialogueHelperProbe>(target);
  if(!value||value->probe_slot!=dialogue_helper_turn)return;
  auto* p=value->output();using D=epok::DialogueSessionLibrary;using R=epok::DialogueResult;using P=epok::DialoguePhase;
  auto* view=epok::active_object_registry->resolve<epok::DialoguePresenter>(value->presenter);
  auto* body=view?epok::interaction::actor(view->body):nullptr;
  auto* first=view?epok::interaction::actor(view->choice0):nullptr;
  if(body){p[21]|=body->data()->text.enabled&&body->data()->text.value[0]=='H';p[23]=body->data()->rect.position[0].raw()==24*4096&&body->data()->rect.position[1].raw()==104*4096&&body->data()->rect.size[0].raw()==272*4096&&body->data()->rect.size[1].raw()==32*4096;}
  if(first)p[22]|=first->data()->text.enabled&&first->data()->text.value[0]=='O';
  if(phase==0){p[1]=uint32_t(value->availability(owner_id()));value->allowed=false;p[2]=uint32_t(value->availability(owner_id()));value->allowed=true;const auto result=value->begin_dialogue(owner_id());handle=result.session;p[3]=uint32_t(result.result);phase=1;return;}
  if(phase==1){const auto state=D::snapshot(handle);if(state.node==LATE)p[20]=state.glyphs==6&&epok::dialogue::session().text()[0]=='R';if(state.phase==P::Selecting){if(!selecting_frames++)return;p[10]=state.choices;value->select_page(handle);p[11]=D::snapshot(handle).choice_page;p[12]=D::choice_id(handle,0)==OPTION;value->advance_session(handle);}else if(state.phase==P::Revealing||state.phase==P::Waiting)value->advance_session(handle);if(!D::valid(handle)){p[9]=uint32_t(D::snapshot(handle).result);p[13]=!epok::GameplayInputLibrary::valid(epok::GameplayInputLibrary::owner_capture(target,epok::GameplayInputLibrary::all_actions(),0));old=handle;phase=2;}return;}
  if(phase==2){handle=value->begin_dialogue(owner_id()).session;p[14]=value->cancel_session(old)==R::NoSession&&value->advance_session(old)==R::NoSession&&D::valid(handle);value->cancel_session(handle);phase=3;return;}
  if(phase==3){handle=value->begin_dialogue(owner_id()).session;if(body)body->set_active(false);phase=4;return;}
  if(phase==4&&!D::valid(handle)){dialogue_focus_probe[2]|=!epok::hud_stats.rectangles&&!epok::hud_stats.glyphs;p[15]=uint32_t(D::snapshot(handle).result);p[17]=!epok::GameplayInputLibrary::valid(epok::GameplayInputLibrary::owner_capture(target,epok::GameplayInputLibrary::all_actions(),0));if(body)body->set_active(true);p[0]=0x444c4833;++dialogue_helper_turn;phase=5;}
 }
};
'''
    for key, name in {"BASE": "base", "CPP": "cpp", "DRIVER": "driver", "CAN_INTERACT": "can-interact",
                      "CONDITION": "condition", "ACTION": "award", "STARTED": "started", "BEGIN": "begin",
                      "WIDGET_BASE": "widget-base", "CPP_WIDGET": "widget-cpp"}.items():
        header = header.replace(f'"{key}"', f'"{identity(name)}"')
    header = header.replace("OPTION", str(compact("option4")))
    header = header.replace("LATE", str(compact("late-line")))
    write(PROJECT / "assets/scripts/DialogueHelperProbe.hpp", header)
    lua_identities = {"assets/scripts/LuaDialogueHelperProbe.lua": identity("lua")}
    if WIDGET_PROVIDER == "lua":
        write(PROJECT / "assets/scripts/LuaDialogueWidget.lua", '''---@class LuaDialogueWidget : DialogueWidgetProbe
local LuaDialogueWidget = DialogueWidgetProbe:extend()
---@override
function LuaDialogueWidget:dialogue_opened(state)
    LuaDialogueWidget.super.dialogue_opened(self, state)
    self.anchor_probe:set_target(self.anchor_probe.target)
    self:observe_anchor(self.anchor_probe:refresh(), self.anchor_probe:status(), self.anchor_probe:valid_content())
    self:observe_widget(state, 301)
end
---@override
function LuaDialogueWidget:dialogue_closed(state)
    LuaDialogueWidget.super.dialogue_closed(self, state)
    self:observe_widget(state, 302)
end
---@override
function LuaDialogueWidget:present(state)
    LuaDialogueWidget.super.present(self, state)
    self:observe_widget(state, 303)
end
return LuaDialogueWidget
''')
        lua_identities["assets/scripts/LuaDialogueWidget.lua"] = identity("widget-lua")
    write(PROJECT / "ProjectSettings/LuaClasses.epoksettings", {"classes": lua_identities})
    metadata = json.loads(run("--project", PROJECT, "--reflect"))
    widget_class = "epok::DialoguePresenter"
    widget_id = next(c["id"] for c in metadata["classes"] if c["cpp_name"] == widget_class)
    if WIDGET_PROVIDER == "cpp":
        widget_class, widget_id = "CppDialogueWidget", identity("widget-cpp")
    if WIDGET_PROVIDER == "lua":
        widget_class, widget_id = "LuaDialogueWidget", identity("widget-lua")
    if WIDGET_PROVIDER == "blueprint":
        widget_path = PROJECT / "assets/Blueprints/BP_DialogueWidget.epokbp"
        if not widget_path.exists():
            run("--project", PROJECT, "--new-blueprint", "BP_DialogueWidget", "--parent", identity("widget-base"))
        widget = documents.loads(widget_path.read_text(encoding="utf-8"))
        base_widget = next(c for c in metadata["classes"] if c["id"] == identity("widget-base"))
        observer = next(f for f in base_widget["functions"] if f["name"] == "observe_widget")
        presenter = next(c for c in metadata["classes"] if c["cpp_name"] == "epok::DialoguePresenter")
        widget["functions"] = []
        for operation, name in enumerate(("dialogue_opened", "dialogue_closed", "present"), 201):
            function = next(f for f in presenter["functions"] if f["name"] == name)
            entry, parent, observed = (str(uuid.uuid4()) for _ in range(3))
            parameter = {"kind": "parameter", "name": "state"}
            widget["functions"].append({"id": str(uuid.uuid4()), "name": name, "override_id": function["id"],
                "parameters": function["parameters"], "returns": function["returns"], "entry": entry,
                "nodes": [{"id": entry, "kind": {"kind": "entry"}, "inputs": {}, "outputs": {"next": [parent]}},
                          {"id": parent, "kind": {"kind": "call_parent"}, "inputs": {"state": parameter}, "outputs": {"next": [observed]}},
                          {"id": observed, "kind": {"kind": "call", "function": observer["id"]}, "inputs": {"state": parameter,
                           "operation": {"kind": "literal", "value_type": {"kind": "uint32"}, "value": operation}}, "outputs": {}}]})
        write(widget_path, widget)
        widget_class, widget_id = "BP_DialogueWidget", widget["id"]
    base = next(c for c in metadata["classes"] if c["id"] == identity("base"))
    functions = {f["name"]: f for f in base["functions"]}
    library = next(c for c in metadata["function_libraries"] if c["cpp_name"] == "epok::DialogueSessionLibrary")
    operations = {o["name"]: o for o in library["operations"]}
    path = PROJECT / "assets/Blueprints/BP_DialogueHelperProbe.epokbp"
    if not path.exists():
        run("--project", PROJECT, "--new-blueprint", "BP_DialogueHelperProbe", "--parent", identity("base"))
    asset = documents.loads(path.read_text(encoding="utf-8"))

    def graph(name):
        function = functions[name]
        nodes = []
        def node(kind, inputs=None, **fields):
            value = {"id": str(uuid.uuid4()), "kind": {"kind": kind, **fields}, "inputs": inputs or {}, "outputs": {}}
            nodes.append(value)
            return value
        def ref(value):
            return {"kind": "link", "node": value["id"], "pin": "value"}
        def operation(name, values):
            op = operations[name]
            return node("operation", {p["id"]: v for p, v in zip(op["parameters"], values)}, operation=op["id"])
        params = {p["name"]: {"kind": "parameter", "name": p["name"]} for p in function["parameters"]}
        entry = node("entry")
        if name in ("advance_session", "cancel_session", "select_page"):
            arguments = [params["handle"]]
            operation_name = {"advance_session": "advance", "cancel_session": "cancel", "select_page": "move_selection"}[name]
            if name == "select_page":
                arguments.append({"kind": "literal", "value_type": {"kind": "int32"}, "value": 4})
            invoked = operation(operation_name, arguments)
            flow = [entry, invoked, node("return", {"value": ref(invoked)})]
        elif name == "begin_dialogue":
            owner = node("builtin", operation={"kind": "self_object"})
            started = operation("start", [ref(owner), params["initiator"], {"kind": "literal", "value_type": {"kind": "uint32"}, "value": 0}])
            observed = node("call", {"started": ref(started)}, function=functions["observe_start"]["id"])
            returned = node("return", {"value": ref(started)})
            flow = [entry, started, observed, returned]
        else:
            parent = node("call_parent", params)
            flow = [entry, parent]
            if name == "dialogue_started":
                snapshot = operation("snapshot", [params["session"]])
                flow.append(node("call", {"state": ref(snapshot), "handle": params["session"]}, function=functions["observe"]["id"]))
            elif function["returns"]["kind"] == "bool":
                flow.append(node("return", {"value": ref(parent)}))
        for first, second in zip(flow, flow[1:]):
            first["outputs"]["next"] = [second["id"]]
        return {"id": str(uuid.uuid4()), "name": name, "override_id": function["id"],
                "parameters": function["parameters"], "returns": function["returns"], "entry": entry["id"], "nodes": nodes}
    asset["functions"] = [graph(name) for name in ("can_interact", "condition", "award", "dialogue_started", "begin_dialogue", "advance_session", "cancel_session", "select_page")]
    write(path, asset)
    write(PROJECT / "assets/scripts/LuaDialogueHelperProbe.lua", '''---@class LuaDialogueHelperProbe : DialogueHelperProbe
local LuaDialogueHelperProbe = DialogueHelperProbe:extend()
---@override
function LuaDialogueHelperProbe:can_interact(initiator)
    return LuaDialogueHelperProbe.super.can_interact(self, initiator)
end
---@override
function LuaDialogueHelperProbe:condition(initiator)
    return LuaDialogueHelperProbe.super.condition(self, initiator)
end
---@override
function LuaDialogueHelperProbe:award(initiator, port)
    LuaDialogueHelperProbe.super.award(self, initiator, port)
end
---@override
function LuaDialogueHelperProbe:dialogue_started(initiator, port, session)
    LuaDialogueHelperProbe.super.dialogue_started(self, initiator, port, session)
    local state = epok.dialogue.snapshot(session)
    if state.source == self.ref then
        self:observe(state, session)
    end
end
---@override
function LuaDialogueHelperProbe:begin_dialogue(initiator)
    local started = epok.dialogue.start(self.ref, initiator, 0)
    if started.session.generation > 0 and epok.dialogue.valid(started.session) then
        self:observe_start(started)
    end
    return started
end
---@override
function LuaDialogueHelperProbe:advance_session(handle)
    return epok.dialogue.advance(handle)
end
---@override
function LuaDialogueHelperProbe:cancel_session(handle)
    return epok.dialogue.cancel(handle)
end
---@override
function LuaDialogueHelperProbe:select_page(handle)
    return epok.dialogue.move_selection(handle, 4)
end
return LuaDialogueHelperProbe
''')
    write(PROJECT / "ProjectSettings/LuaClasses.epoksettings", {"classes": lua_identities})
    text = lambda key: {"table": identity("strings"), "key": key}
    package("assets/dialogue/strings.epokasset", "StringTable", "strings", {"version": 1, "pageable": True, "languages": ["English"],
            "entries": [{"key": "line", "values": ["Hello"]}, {"key": "prompt", "values": ["Which option would you like? " * 4]}] + [{"key": f"option{i}", "values": [f"Option {i}"]} for i in range(6)]})
    package("assets/dialogue/late-strings.epokasset", "StringTable", "late-strings", {"version": 1, "pageable": True,
            "languages": ["English"], "entries": [{"key": "reward", "values": ["Reward"]}]})
    nodes = [
        {"id": identity("line"), "kind": {"kind": "line", "speaker": None, "text": text("line"), "next": identity("choice"), "policy": None}},
        {"id": identity("choice"), "kind": {"kind": "choice", "speaker": None, "prompt": text("prompt"),
         "options": [{"id": identity(f"option{i}"), "text": text(f"option{i}"), "next": identity("branch"), "condition": None} for i in range(6)]}},
        {"id": identity("branch"), "kind": {"kind": "branch", "condition": identity("condition-binding"), "yes": identity("event"), "no": identity("end")}},
        {"id": identity("event"), "kind": {"kind": "event", "action": identity("award-binding"), "next": identity("late-line")}},
        {"id": identity("late-line"), "kind": {"kind": "line", "speaker": None,
         "text": {"table": identity("late-strings"), "key": "reward"}, "next": identity("end"), "policy": None}},
        {"id": identity("end"), "kind": {"kind": "end"}},
    ]
    package("assets/dialogue/conversation.epokasset", "Dialogue", "conversation", {"version": 1, "entry": identity("line"), "nodes": nodes,
            "participants": [{"id": identity("role"), "label": "Rules", "display_name": None, "portrait": None}],
            "bindings": [{"id": identity(f"{name}-binding"), "label": name, "participant": identity("role"), "kind": kind,
                          "class": identity("base"), "member": identity(name)} for kind, name in (("condition", "condition"), ("action", "award"))]})
    descriptor = next(PROJECT.glob("*.epokproject"))
    settings = documents.loads(descriptor.read_text(encoding="utf-8"))
    scene_path = PROJECT / settings["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    camera = copy.deepcopy(scene["actors"][0])
    for root in camera["components"]:
        if root.get("root"):
            root["properties"].update(position=[0, 0, 0], rotation=[0, 0, 0])
    scene["actors"] = [camera]
    reflected = {c["cpp_name"]: c for c in metadata["classes"]}

    def component(name, key, props=None, root=False):
        props = props or {}
        return {"id": identity(key), "name": key, "class": {"name": name, "class_id": reflected[name]["id"]},
                "properties": props, "overrides": list(props), "root": root}
    def actor(name, class_name, root_name):
        value = copy.deepcopy(camera)
        value.update(id=identity(name), name=name, **{"class": {"name": class_name, "class_id": reflected[class_name]["id"]}}, properties={}, overrides=[])
        value["components"] = [component(root_name, name + ".root", root=True)]
        return value
    view = actor("presenter", "epok::DialoguePresenter", "epok::RectTransformComponent")
    view["class"] = {"name": widget_class, "class_id": widget_id}
    view["components"][0]["properties"] = {"rect": {"anchor_min": [0, 0], "anchor_max": [1, 1], "pivot": [0, 0], "position": [0, 0], "size": [0, 0], "rotation": 0}}
    view["components"][0]["overrides"] = ["rect"]
    view["components"] += [component("epok::CanvasComponent", "canvas"), component("epok::AudioComponent", "audio")]
    for inherited, default_id in zip(view["components"][1:], ("0762122d-cb6b-52f6-8fb1-3287494801fc", "b0e900ca-43e6-5bdf-bdbd-a52f0416b36b")):
        inherited.update(inherited=True, default_id=default_id)
    for field in ("panel", "portrait", "speaker_name", "body", "prompt", "continuation", "choice0", "choice1", "choice2", "choice3"):
        child = actor(field, "epok::UIActor", "epok::RectTransformComponent")
        child["logical_parent"] = identity("presenter")
        child["attach"] = {"actor": identity("presenter"), "component": identity("presenter.root")}
        rect = {"panel": [12, 24, 296, 200], "portrait": [20, 48, 48, 64], "prompt": [24, 152, 272, 32],
                "continuation": [280, 28, 8, 16], "speaker_name": [24, 200, 272, 16], "body": [24, 104, 272, 32]}.get(field)
        if rect is None:
            rect = [24, 40 + (3 - int(field[-1])) * 24, 272, 24]
        child["components"][0]["properties"] = {"rect": {"anchor_min": [0, 0], "anchor_max": [0, 0], "pivot": [0, 0], "position": rect[:2], "size": rect[2:], "rotation": 0}}
        child["components"][0]["overrides"] = ["rect"]
        kind = "epok::ImageComponent" if field in ("panel", "portrait") else "epok::TextComponent"
        label = component(kind, field + ".label", {"text": {"enabled": False, "text": ""}} if kind == "epok::TextComponent" else {})
        if field == "continuation":
            label["properties"]["text"] = {"enabled": False, "text": ">", "color": [1, 1, 1]}
        if kind == "epok::ImageComponent":
            label["properties"]["image"] = {"enabled": False, "color": [0.10, 0.14, 0.24] if field == "panel" else [1, 1, 1],
                                                 "region": [0, 0, 0, 0], "borders": [0, 0, 0, 0], "tiling": "none"}
            label["overrides"].append("image")
        child["components"].append(label)
        if field.startswith("choice"):
            child["components"].append(component("epok::FocusableComponent", field + ".focus", {
                "focusable": {"enabled": True, "outline": True, "highlight": [1, .9, .3]}}))
        view["properties"][field] = label["id"]
        scene["actors"].append(child)
    if WIDGET_PROVIDER != "native":
        view["properties"]["anchor_probe"] = identity("anchor_component")
    view["overrides"] = list(view["properties"])
    scene["actors"].append(view)
    scene["actors"].append(actor("projection", "ProjectionProbe", "epok::SceneComponent3D"))
    anchor_view = actor("anchor_canvas", "epok::UIActor", "epok::RectTransformComponent")
    anchor_view["components"].append(component("epok::CanvasComponent", "anchor_canvas.canvas"))
    anchor_content = actor("anchor_content", "epok::UIActor", "epok::RectTransformComponent")
    anchor_content["logical_parent"] = anchor_view["id"]
    anchor_content["attach"] = {"actor": anchor_view["id"], "component": identity("anchor_canvas.root")}
    anchor_content["components"][0]["properties"] = {"rect": {"anchor_min": [0, 0], "anchor_max": [0, 0], "pivot": [.5, 0], "position": [0, 0], "size": [100, 40], "rotation": 0}}
    anchor_content["components"][0]["overrides"] = ["rect"]
    anchor_probe = actor("anchor_probe", "AnchorProbe", "epok::SceneComponent3D")
    anchor_probe["components"][0]["properties"] = {"position": [.5, 0, 2], "rotation": [0, 0, 0]}
    anchor_probe["components"][0]["overrides"] = ["position", "rotation"]
    anchor = component("epok::ScreenAnchorComponent", "anchor_component", {"content": identity("anchor_content.root"), "target": anchor_probe["id"]})
    anchor_view["components"].append(anchor)
    anchor_probe["properties"] = {"anchor": anchor["id"]}
    anchor_probe["overrides"] = ["anchor"]
    scene["actors"] += [anchor_view, anchor_content, anchor_probe]
    for domain in (3, 2):
        for index, (name, class_id) in enumerate((("CppDialogueHelperProbe", identity("cpp")), ("BP_DialogueHelperProbe", asset["id"]), ("LuaDialogueHelperProbe", identity("lua")))):
            slot = (3 - domain) * 3 + index
            source = actor(f"source{slot}", f"epok::Actor{domain}D", f"epok::SceneComponent{domain}D")
            target = actor(f"target{slot}", f"epok::Actor{domain}D", f"epok::SceneComponent{domain}D")
            endpoint_id = identity(f"endpoint{slot}")
            bindings = component("epok::DialogueRoleBindingsComponent", f"bindings{slot}", {"role0": compact("role"), "target0": endpoint_id})
            props = {"probe_slot": slot, "conversation": identity("conversation"), "presenter": view["id"], "bindings": bindings["id"], "automatic_input": False}
            target["components"] += [bindings, {"id": endpoint_id, "name": name, "class": {"name": name, "class_id": class_id}, "properties": props, "overrides": list(props)}]
            source["components"].append(component("DialogueHelperDriver", f"driver{slot}", {"target": endpoint_id}))
            scene["actors"] += [source, target]
    scene.pop("bake", None)
    write(scene_path, scene)
    run("--project", PROJECT, "--compile-blueprints", log="compile.log")
    return descriptor


def emulate(mode):
    with socket.socket() as probe:
        if os.name == "nt":
            probe.setsockopt(socket.SOL_SOCKET, socket.SO_EXCLUSIVEADDRUSE, 1)
        probe.bind(("127.0.0.1", 8077))
    expected = [MAGIC, 0, 1, 0, 3, 3, 8, 1, 3, 8, 6, 1, 1, 1, 1, 11, 11, 1, 3, 1, 1, 1, 1, 1] * 6
    with (OUT / f"{mode}-play.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen([str(EDITOR), "--project", str(PROJECT), "--play-psx", "--stop-after", "6"], cwd=ROOT,
                                   stdout=log, stderr=log, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        values = None
        widget_values = None
        projection_values = None
        focus_values = None
        anchor_values = None
        anchor_vm_values = None
        def widget_ready():
            if WIDGET_PROVIDER == "native":
                return True
            marker = {"cpp": 1, "blueprint": 2, "lua": 3}[WIDGET_PROVIDER]
            return (widget_values is not None and widget_values[:2] == [18, 18] and widget_values[2] > 0
                    and widget_values[3:] == [18, 18, 1, 1, 0, marker, 18])
        try:
            deadline = time.monotonic() + 120
            while process.poll() is None and time.monotonic() < deadline:
                try:
                    with urllib.request.urlopen("http://127.0.0.1:8077/api/v1/cpu/ram/raw", timeout=2) as response:
                        ram = response.read()
                    # Play may restage/relink after the analysis build. Resolve
                    # the symbol from the map belonging to the running image.
                    mapping = (PROJECT / ".epok/build/epok.map").read_text()
                    address = int(re.findall(r"0x([0-9a-fA-F]+)\s+dialogue_helper_probe\s*$", mapping, re.M)[-1], 16) & 0x1fffff
                    values = list(struct.unpack_from(f"<{WORDS * 6}I", ram, address))
                    if WIDGET_PROVIDER != "native":
                        address = int(re.findall(r"0x([0-9a-fA-F]+)\s+dialogue_widget_probe\s*$", mapping, re.M)[-1], 16) & 0x1fffff
                        widget_values = list(struct.unpack_from("<10I", ram, address))
                    address = int(re.findall(r"0x([0-9a-fA-F]+)\s+camera_projection_probe\s*$", mapping, re.M)[-1], 16) & 0x1fffff
                    projection_values = list(struct.unpack_from("<2I", ram, address))
                    address = int(re.findall(r"0x([0-9a-fA-F]+)\s+dialogue_focus_probe\s*$", mapping, re.M)[-1], 16) & 0x1fffff
                    focus_values = list(struct.unpack_from("<4I", ram, address))
                    address = int(re.findall(r"0x([0-9a-fA-F]+)\s+screen_anchor_probe\s*$", mapping, re.M)[-1], 16) & 0x1fffff
                    anchor_values = list(struct.unpack_from("<2I", ram, address))
                    if WIDGET_PROVIDER == "lua":
                        address = int(re.findall(r"0x([0-9a-fA-F]+)\s+dialogue_anchor_vm_probe\s*$", mapping, re.M)[-1], 16) & 0x1fffff
                        anchor_vm_values = list(struct.unpack_from("<3I", ram, address))
                    if values == expected and widget_ready() and projection_values == [16, 16] and anchor_values == [8, 8] and (WIDGET_PROVIDER != "lua" or anchor_vm_values == [18, 18, 18]) and (WIDGET_PROVIDER == "native" or focus_values == [1, 1, 1, 1]):
                        break
                except (OSError, ValueError):
                    pass
                time.sleep(.1)
            assert values == expected, {"mode": mode, "actual": values, "expected": expected}
            assert widget_ready(), {"mode": mode, "widget_provider": WIDGET_PROVIDER, "widget": widget_values}
            assert projection_values == [16, 16], {"mode": mode, "projection": projection_values}
            assert anchor_values == [8, 8], {"mode": mode, "anchor": anchor_values}
            assert WIDGET_PROVIDER != "lua" or anchor_vm_values == [18, 18, 18], {"mode": mode, "anchor_vm": anchor_vm_values}
            assert WIDGET_PROVIDER == "native" or focus_values == [1, 1, 1, 1], {"mode": mode, "focus": focus_values}
            assert process.wait(timeout=30) == 0
        finally:
            if process.poll() is None:
                if os.name == "nt":
                    subprocess.run(["taskkill.exe", "/PID", str(process.pid), "/T", "/F"], capture_output=True, check=False)
                else:
                    process.terminate()
                process.wait(timeout=15)
    return values, widget_values, projection_values, focus_values, anchor_values, anchor_vm_values


def standalone():
    export = Path(run("--project", PROJECT, "--export-psx", log="export.log").strip())
    env = os.environ.copy()
    env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
    rebuilt = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
                              "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")],
                             cwd=export, env=env, capture_output=True, text=True, timeout=900)
    (OUT / "standalone-build.log").write_text(rebuilt.stdout + rebuilt.stderr, encoding="utf-8")
    assert rebuilt.returncode == 0, rebuilt.stdout + rebuilt.stderr
    original = (PROJECT / ".epok/build/epok.ps-exe").read_bytes()
    assert (export / "epok.ps-exe").read_bytes() == original
    manifest = documents.loads(next(PROJECT.glob("*.epokproject")).read_text(encoding="utf-8"))
    result = {"mode": manifest["lua_execution"], "export": str(export), "identical": True,
              "editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(), "sha256": hashlib.sha256(original).hexdigest()}
    (OUT / "standalone-result.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print("PASS standalone dialogue helpers rebuild, byte-identical", manifest["lua_execution"], flush=True)


def main():
    global OUT, PROJECT, WIDGET_PROVIDER
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, help="Evidence directory; keeps earlier phase results intact")
    parser.add_argument("--widget-provider", choices=("native", "cpp", "blueprint", "lua"), default="native")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--standalone-only", action="store_true")
    parser.add_argument("--mode", action="append", choices=("native_cpp", "vm_bytecode", "vm_source"))
    args = parser.parse_args()
    WIDGET_PROVIDER = args.widget_provider
    if args.output:
        OUT = args.output.resolve()
        PROJECT = OUT / "project"
    if args.standalone_only:
        standalone()
        return
    descriptor = prepare()
    if args.prepare_only:
        print("PASS dialogue helpers fixture and provider compilation", flush=True)
        return
    path = OUT / "result.json"
    results = json.loads(path.read_text()) if args.mode and path.exists() else []
    for mode in args.mode or ("native_cpp", "vm_bytecode", "vm_source"):
        settings = documents.loads(descriptor.read_text(encoding="utf-8"))
        settings["lua_execution"] = mode
        write(descriptor, settings)
        run("--project", PROJECT, "--analyze-memory", log=f"{mode}-build.log")
        values, widget, projection, focus, anchor, anchor_vm = emulate(mode)
        results = [r for r in results if r["mode"] != mode]
        build = PROJECT / ".epok/build"
        results.append({"mode": mode, "widget_provider": WIDGET_PROVIDER, "values": values, "widget": widget, "camera_projection": projection, "focus": focus, "anchor": anchor, "anchor_vm": anchor_vm, "memory": json.loads((build / "memory-report.json").read_text())["ram"]["used"],
                        "editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(), "exe_sha256": hashlib.sha256((build / "epok.ps-exe").read_bytes()).hexdigest(),
                        "runtime_sources": {name: hashlib.sha256((build / name).read_bytes()).hexdigest() for name in ("dialogue.hpp", "dialogue_helpers.hpp", "interaction.hpp", "gameplay_control.hpp", "epok.hpp", "object_model.hpp", "hud_core.hpp", "hud.hpp", "camera_projection.hpp", "ui_anchor.hpp", "ui_anchor_service.hpp")}})
        path.write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")
        print("PASS", mode, "C++/Blueprint/Lua dialogue sessions, role callbacks and lifecycle in 2D/3D", flush=True)


if __name__ == "__main__":
    main()
