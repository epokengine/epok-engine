// Observable routing, identity and lifecycle contracts over the real actor model.
#define EPOK_GAMEPLAY_CONTROL 1
#include <cassert>
#include <cstdio>
#include "../../runtime/object_model.hpp"

namespace epok {
// This lifecycle fixture creates object identities without spatial data slots.
int entity_index(const ActorData*){return -1;}
void AudioSource::play(){}
void AudioSource::stop(){}
bool AudioSource::is_playing() const{return false;}
const ClassDescriptor object_classes[]={
    {Object::static_class_id,0,ObjectFamily::Object,ObjectDomain::None,0,ObjectClassAbstract,nullptr,nullptr,sizeof(Object),alignof(Object),nullptr,nullptr},
    {Actor::static_class_id,Object::static_class_id,ObjectFamily::Actor,ObjectDomain::None,0,ObjectClassAbstract,nullptr,nullptr,sizeof(Actor),alignof(Actor),nullptr,nullptr},
    {ActorComponent::static_class_id,Object::static_class_id,ObjectFamily::Component,ObjectDomain::None,7,ObjectClassAbstract,nullptr,nullptr,sizeof(ActorComponent),alignof(ActorComponent),nullptr,nullptr},
    {Actor3D::static_class_id,Actor::static_class_id,ObjectFamily::Actor,ObjectDomain::World3D,0,uint8_t(ObjectClassPlaceable|ObjectClassSpawnable),&object_construct<Actor3D>,&object_destruct,sizeof(Actor3D),alignof(Actor3D),&ObjectPool<Actor3D,4>::acquire,&ObjectPool<Actor3D,4>::release},
    {SceneComponent3D::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,object_domain_bit(ObjectDomain::World3D),ObjectClassRoot,&object_construct<SceneComponent3D>,&object_destruct,sizeof(SceneComponent3D),alignof(SceneComponent3D),nullptr,nullptr},
    {Level::static_class_id,Object::static_class_id,ObjectFamily::Level,ObjectDomain::None,0,ObjectClassAbstract,nullptr,nullptr,sizeof(Level),alignof(Level),nullptr,nullptr},
};
const size_t object_class_count=sizeof(object_classes)/sizeof(object_classes[0]);
}
using namespace epok;

int main(){
    auto& pad=epok::input;pad.reset();
    auto& controls=gameplay_controls();controls.reset();
    ObjectRegistryStorage<32> registry;
    active_object_registry=&registry;
    Level level;assert(level.bind(registry));
    auto parent=level.spawn_actor(*find_object_class(Actor3D::static_class_id),"Parent");
    auto owner=level.spawn_actor(*find_object_class(Actor3D::static_class_id),"Owner",parent);
    assert(parent.valid()&&owner.valid());
    const auto confirm=GameplayControls::mask(GameplayAction::Confirm);
    auto frame=[&](uint16_t buttons,unsigned port=0){pad.sample(port,true,buttons);controls.begin_frame();};
    auto tick=[&](){pad.begin_tick();controls.begin_tick();};
    constexpr uint16_t cross=1u<<unsigned(Button::Cross);

    // A short press/release during two zero-step frames is retained once.
    frame(cross);frame(0);tick();
    auto value=controls.sample(GameplayAction::Jump,0);
    assert(value.pressed&&value.released&&!value.held&&value.connected);
    assert(controls.take_pressed(GameplayAction::Interact,0));
    assert(!controls.take_pressed(GameplayAction::Jump,0)); // shared physical edge
    controls.end_tick();tick();assert(!controls.sample(GameplayAction::Jump,0).pressed);

    // Frame consumption cannot be replayed in a later simulation step.
    frame(cross);assert(controls.take_pressed(GameplayAction::Confirm,0,true));
    tick();assert(!controls.sample(GameplayAction::Jump,0).pressed);
    assert(pad.pressed(Button::Cross)); // raw Input retains its independent contract

    // The opening edge is suppressed, and physical aliases cannot bypass capture.
    auto first=controls.capture(owner,confirm,100,0);assert(controls.valid(first));
    assert(controls.owner_capture(owner,confirm,0).generation==first.generation);
    assert(!controls.valid(controls.owner_capture(parent,confirm,0)));
    assert(!controls.context_sample(first,GameplayAction::Confirm,false).pressed);
    assert(!controls.sample(GameplayAction::Jump,0).held);
    assert(!controls.button_held(Button::Cross,0));
    frame(0);frame(cross);tick();
    assert(controls.context_sample(first,GameplayAction::Confirm,false).pressed);
    assert(!controls.context_sample(first,GameplayAction::Jump,false).pressed);
    assert(!controls.sample(GameplayAction::Jump,0).pressed);
    auto background=controls.capture(parent,confirm,50,0);
    assert(controls.context_sample(first,GameplayAction::Confirm,false).pressed);
    assert(controls.release(background));
    assert(controls.context_sample(first,GameplayAction::Confirm,false).pressed);
    assert(controls.context_take_pressed(first,GameplayAction::Confirm,false));
    assert(!controls.context_take_pressed(first,GameplayAction::Confirm,true));

    // Priority, newest-wins ties, independent ports and out-of-order release.
    auto lower=controls.capture(parent,confirm,50,0);
    auto newer=controls.capture(parent,confirm,100,0);
    auto other_port=controls.capture(parent,confirm,200,1);
    frame(0);frame(cross);tick();
    assert(!controls.context_sample(first,GameplayAction::Confirm,false).held);
    assert(controls.context_sample(newer,GameplayAction::Confirm,false).held);
    assert(controls.release(lower)&&controls.valid(first)&&controls.valid(newer));
    assert(controls.release(newer)&&controls.valid(first)&&controls.valid(other_port));
    assert(controls.context_sample(first,GameplayAction::Confirm,false).held);
    assert(controls.release(first)&&controls.release(other_port));
    assert(!controls.sample(GameplayAction::Jump,0).pressed); // closing edge cannot jump

    // Capturing movement removes commands but does not mutate pad/physics state.
    frame(0);pad.sample(0,true,0,true,255,128,255,128);controls.begin_frame();
    assert(controls.axis_raw(Axis::LeftX,0)==4096);
    auto move=controls.capture(owner,GameplayInputLibrary::movement_actions(),100,0);
    assert(controls.axis_raw(Axis::LeftX,0)==0&&pad.axis_raw(Axis::LeftX)==4096);
    assert(controls.sample(GameplayAction::MoveX,0).value.raw()==0);
    assert(controls.axis_raw(Axis::RightX,0)==4096);
    assert(controls.release(move));

    // Inherited disable, component owner, destruction and token generations.
    auto root=registry.resolve<Actor>(owner)->root_id();
    auto component_capture=controls.capture(root,confirm,100,0);assert(controls.valid(component_capture));
    assert(level.set_active(parent,false));assert(!controls.valid(component_capture));
    assert(level.set_active(parent,true));
    auto transient=controls.capture(owner,confirm,100,0);
    assert(level.set_active(parent,false)&&level.set_active(parent,true));
    assert(!controls.valid(transient)); // No routed read while the parent was inactive.
    transient=controls.capture(owner,confirm,100,0);
    assert(level.set_active(owner,false)&&level.set_active(owner,true));
    assert(!controls.valid(transient)); // Inert-leaf activation fast path too.
    assert(level.set_logical_parent(owner,{}));
    assert(level.set_active(parent,false));
    transient=controls.capture(owner,confirm,100,0);assert(controls.valid(transient));
    assert(level.set_logical_parent(owner,parent)&&level.set_logical_parent(owner,{}));
    assert(!controls.valid(transient)); // Reparenting through an inactive ancestor.
    assert(level.set_active(parent,true)&&level.set_logical_parent(owner,parent));
    auto alive=controls.capture(owner,confirm,100,0);assert(controls.valid(alive));
    assert(level.destroy_actor(owner));assert(!controls.valid(alive));
    auto replacement=level.spawn_actor(*find_object_class(Actor3D::static_class_id),"Replacement");
    auto current=controls.capture(replacement,confirm,100,0);assert(controls.valid(current));
    assert(!controls.valid(alive)&&!controls.release(alive));
    controls.reset();assert(!controls.valid(current));
    auto after_reset=controls.capture(replacement,confirm,100,0);
    assert(controls.valid(after_reset)&&after_reset.generation!=current.generation);
    controls.release(after_reset);

    // Exhaustion and bad bindings fail without releasing another capture.
    InputCapture full[16];
    for(auto& token:full){token=controls.capture(parent,confirm,1,0);assert(controls.valid(token));}
    assert(!controls.valid(controls.capture(parent,confirm,2,0)));
    for(auto token:full)assert(controls.release(token));
    assert(!controls.valid(controls.capture({},confirm,1,0)));
    assert(!controls.valid(controls.capture(parent,0,1,0)));
    assert(!controls.valid(controls.capture(parent,0x80000000u,1,0)));
    assert(!controls.valid(controls.capture(parent,confirm,1,4)));
    assert(!controls.bind_button(GameplayAction::Count,Button::Cross,0));
    assert(!controls.bind_axis(GameplayAction::MoveX,Axis(4),Button::Left,Button::Right,0));
    assert(controls.bind_button(GameplayAction::Custom0,Button::Square,0));
    frame(1u<<unsigned(Button::Square));tick();
    assert(controls.take_pressed(GameplayAction::Custom0,0));
    assert(!controls.sample(GameplayAction::Attack,0).pressed);

    // Paused UI consumes frame input; resuming cannot see an old tick edge.
    frame(0);frame(cross);controls.discard_tick_edges();
    assert(controls.take_pressed(GameplayAction::Confirm,0,true));
    tick();assert(!controls.sample(GameplayAction::Confirm,0).pressed);
    auto unloading=controls.capture(parent,confirm,100,0);assert(controls.valid(unloading));
    level.end_play_all(EndPlayReason::LevelUnloaded);assert(!controls.valid(unloading));
    registry.release(level.id());active_object_registry=nullptr;
    std::printf("Gameplay controls: edges, aliases, priorities, lifecycle, binding and bounds pass; storage %zu bytes.\n",sizeof(GameplayControls));
}
