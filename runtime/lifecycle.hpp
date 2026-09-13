#pragma once
#ifdef EPOK_BLUEPRINTS
#include "blueprint_spawn.hpp"
#endif
// Included after generated scene tables. Storage never moves; handles carry a
// generation so destroying/reusing a slot cannot revive a reference.
namespace epok {
inline bool lifecycle_tearing_down=false;
inline int entity_index(const Entity* entity) {
    if(!entity)return -1;
    // Slots are contiguous and never move. Integer range/alignment checks also
    // reject foreign pointers without undefined pointer subtraction.
    const uintptr_t address=reinterpret_cast<uintptr_t>(entity);
    const uintptr_t first=reinterpret_cast<uintptr_t>(objects.data());
    if(address<first)return -1;
    const uintptr_t offset=address-first;
    if(offset>=object_count*sizeof(Entity)||offset%sizeof(Entity))return -1;
    return int(offset/sizeof(Entity));
}
inline uint32_t next_generation(uint32_t n){++n;return n?n:1;}
EntityHandle handle(const Entity* entity) {
    int i=entity_index(entity);
    return i<0||!entity->alive?EntityHandle{}:EntityHandle{uint16_t(i),entity->generation};
}
Entity* EntityHandle::get() const {
    return index<object_count&&objects[index].alive&&objects[index].generation==generation?&objects[index]:nullptr;
}
bool is_active_slot(size_t index) {
    int i=int(index);
    for(size_t depth=0;i>=0&&depth<=objects.size();++depth){
        if(size_t(i)>=object_count||!objects[i].alive||!objects[i].active)return false;
        i=objects[i].parent;
        if(i<0)return true;
    }
    return false;
}
bool is_active(const Entity* entity) {
    int i=entity_index(entity);
    return i>=0&&is_active_slot(size_t(i));
}
inline bool descendant(size_t child,size_t ancestor){
    for(size_t depth=0;child<object_count&&depth<=objects.size();++depth){
        if(child==ancestor)return true;
        int p=objects[child].parent;if(p<0)return false;child=size_t(p);
    }
    return false;
}
bool set_active(Entity* entity,bool active) {
    int i=entity_index(entity);if(i<0||!entity->alive)return false;
    bool before[objects.size()]={};
#ifdef EPOK_BLUEPRINTS
    EntityHandle before_owner[objects.size()]={};
    for(size_t n=0;n<object_count;++n)before_owner[n]=handle(&objects[n]);
#endif
    for(size_t n=0;n<object_count;++n)before[n]=is_active(&objects[n]);
    entity->active=active;
    // Audio is a component; entities do not need a script binding to be stopped.
    for(size_t n=0;n<object_count;++n)
        if(before[n]&&!is_active(&objects[n]))objects[n].audio.stop();
#ifdef EPOK_EDITOR_PREVIEW
    if(editor_preview_active)return true;
#endif
    for(auto& b:bindings)if(b.entity<object_count&&objects[b.entity].alive){
        bool after=is_active(&objects[b.entity]);
        if(before[b.entity]&&!after)b.behaviour->on_disable();
        else if(!before[b.entity]&&after)b.behaviour->on_enable();
    }
#ifdef EPOK_BLUEPRINTS
    bp::visit([&](Binding& b,EntityHandle owner){
        if(!bp::same_owner(before_owner[b.entity],owner))return;
        bool after=is_active(owner.get());
        if(before[b.entity]&&!after)b.behaviour->on_disable();
        else if(!before[b.entity]&&after)b.behaviour->on_enable();
    });
#endif
    return true;
}
#ifdef EPOK_BLUEPRINTS
void bp::activate_spawn_audio(EntityHandle root) {
    if(!root.get())return;
    for(size_t i=0;i<object_count;++i)if(descendant(i,root.index)&&is_active_slot(i)){
        auto& audio=objects[i].audio;
        if(audio.enabled&&audio.play_on_start&&!audio.is_playing())audio.play();
    }
}
#endif
#ifdef EPOK_BLUEPRINTS
Behaviour* bp::authored_behaviour(EntityHandle owner) {
    if(!owner.get())return nullptr;
    for(const auto& binding:bindings)if(binding.entity==owner.index)return binding.behaviour;
    return nullptr;
}
bp::ClassId bp::authored_class(EntityHandle owner) {
    if(!owner.get())return 0;
    for(const auto& binding:bindings)if(binding.entity==owner.index)return binding.class_id?binding.class_id:binding.behaviour->blueprint_class_id();
    return 0;
}
#endif
bool destroy_entity(Entity* entity) {
    int index=entity_index(entity);if(index<0||!entity->alive)return false;
    bool doomed[objects.size()]={},was_active[objects.size()]={};
    for(size_t i=0;i<object_count;++i){doomed[i]=objects[i].alive&&descendant(i,size_t(index));was_active[i]=is_active(&objects[i]);}
    // Mark first so reentrant callbacks cannot destroy an object twice.
    for(size_t i=0;i<object_count;++i)if(doomed[i]){
        remove_runtime_owner(i);
        objects[i].audio.stop();objects[i].alive=false;objects[i].active=false;
        objects[i].generation=next_generation(objects[i].generation);
    }
#ifdef EPOK_EDITOR_PREVIEW
    if(editor_preview_active)return true;
#endif
#ifdef EPOK_BLUEPRINTS
    bp::retire(doomed,was_active,object_count);
#endif
    for(auto& b:bindings)if(b.entity<object_count&&doomed[b.entity]){
#ifdef EPOK_BLUEPRINTS
        b.behaviour->blueprint_cancel();
#endif
        if(was_active[b.entity])b.behaviour->on_disable();b.behaviour->on_destroy();
    }
    return true;
}
Entity* find_entity(const char* name){
    if(!name)return nullptr;
    for(size_t i=0;i<object_count;++i)if(objects[i].alive){
        size_t c=0;while(name[c]&&objects[i].name[c]==name[c])++c;
        if(!name[c]&&!objects[i].name[c])return &objects[i];
    }
    return nullptr;
}
Entity* create_entity(const char* name,Entity* parent){
    if(lifecycle_tearing_down)return nullptr;
    int p=entity_index(parent);if(parent&&(p<0||!parent->alive))return nullptr;
    int depth=0;for(int i=p;i>=0;i=objects[i].parent)if(size_t(i)>=object_count||!objects[i].alive||++depth>32)return nullptr;
    // Authored slots retain script bindings and are reused only on scene reset.
    size_t index=authored_count;
    // XA lookup/stop callbacks retain the AudioSource address asynchronously.
    // Quarantine that slot until the existing CD owner releases it.
    for(;index<object_count;++index)if(!objects[index].alive&&music_active!=&objects[index].audio&&music_requested!=&objects[index].audio
#ifdef EPOK_BLUEPRINTS
        &&!bp::slot_quarantined(index)
#endif
    )break;
    if(index>=objects.size())return nullptr;
    if(index==object_count)++object_count;
    auto generation=objects[index].generation;
    auto& e=objects[index];e=Entity{};e.generation=generation;e.parent=p;
    e.transform.scale[0]=e.transform.scale[1]=e.transform.scale[2]=1.0;
    e.material.color[0]=e.material.color[1]=e.material.color[2]=255;e.set_name(name);
    return &e;
}
}
