#pragma once
#ifdef EPOK_BLUEPRINTS
#include "blueprint_spawn.hpp"
#endif
// Included after generated scene tables. Storage never moves; handles carry a
// generation so destroying/reusing a slot cannot revive a reference.
namespace epok {
inline bool lifecycle_tearing_down=false;
inline int entity_index(const ActorData* entity) {
    if(!entity)return -1;
    // Slots are contiguous and never move. Integer range/alignment checks also
    // reject foreign pointers without undefined pointer subtraction.
    const uintptr_t address=reinterpret_cast<uintptr_t>(entity);
    const uintptr_t first=reinterpret_cast<uintptr_t>(objects.data());
    if(address<first)return -1;
    const uintptr_t offset=address-first;
    if(offset>=object_count*sizeof(ActorData)||offset%sizeof(ActorData))return -1;
    return int(offset/sizeof(ActorData));
}
inline uint32_t next_generation(uint32_t n){++n;return n?n:1;}
DataHandle handle(const ActorData* entity) {
    int i=entity_index(entity);
    return i<0||!entity->alive?DataHandle{}:DataHandle{uint16_t(i),entity->generation};
}
ActorData* DataHandle::get() const {
    return index<object_count&&objects[index].alive&&objects[index].generation==generation?&objects[index]:nullptr;
}
bool is_active_slot(size_t index) {
    if(index<object_count && objects[index].owner && active_object_registry) {
        auto* owner=objects[index].owner;
        auto* level=active_object_registry->resolve<Level>(owner->level_id());
        return objects[index].alive && level && level->actor_active(*owner);
    }
    int i=int(index);
    for(size_t depth=0;i>=0&&depth<=objects.size();++depth){
        if(size_t(i)>=object_count||!objects[i].alive||!objects[i].active)return false;
        i=objects[i].parent;
        if(i<0)return true;
    }
    return false;
}
bool is_active(const ActorData* entity) {
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
bool set_active(ActorData* data,bool active) {
    if(!data||!data->owner||!active_object_registry)return false;
    auto* level=active_object_registry->resolve<Level>(data->owner->level_id());
    if(!level)return false;
    data->active=active;
    return level->set_active(data->owner->id(),active);
}
bool destroy_actor_data(ActorData* data) {
    if(!data||!data->owner||!active_object_registry)return false;
    auto* level=active_object_registry->resolve<Level>(data->owner->level_id());
    return level && level->destroy_actor(data->owner->id());
}
ActorData* find_actor_data(const char* name){
    if(!name)return nullptr;
    for(size_t i=0;i<object_count;++i)if(objects[i].alive){
        size_t c=0;while(name[c]&&objects[i].name[c]==name[c])++c;
        if(!name[c]&&!objects[i].name[c])return &objects[i];
    }
    return nullptr;
}
ActorData* allocate_actor_data(const char* name,ActorData* parent){
    if(lifecycle_tearing_down)return nullptr;
    int p=entity_index(parent);if(parent&&(p<0||!parent->alive))return nullptr;
    int depth=0;for(int i=p;i>=0;i=objects[i].parent)if(size_t(i)>=object_count||!objects[i].alive||++depth>32)return nullptr;
    // Authored slots retain script bindings and are reused only on scene reset.
    size_t index=authored_count;
    // XA lookup/stop callbacks retain the AudioSource address asynchronously.
    // Quarantine that slot until the existing CD owner releases it.
    for(;index<object_count;++index)if(!objects[index].alive&&music_active!=&objects[index].audio&&music_requested!=&objects[index].audio
    )break;
    if(index>=objects.size())return nullptr;
    if(index==object_count)++object_count;
    auto generation=objects[index].generation;
    auto& e=objects[index];e=ActorData{};e.generation=generation;e.parent=p;
    e.transform.scale[0]=e.transform.scale[1]=e.transform.scale[2]=1.0;
    e.material.color[0]=e.material.color[1]=e.material.color[2]=255;e.set_name(name);
    return &e;
}
}
