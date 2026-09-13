#pragma once
#include "blueprint_spawn.hpp"
namespace epok::bp {
struct TemplateBinding {
    uint16_t entity = 0;
    ClassId class_id = 0;
    void (*apply)(Behaviour*,const EntityHandle*,size_t) = nullptr;
};
// Prototypes are the host-constructed, normally cooked Object records. Their
// resource pointers reference immutable generated data, never editor addresses.
// Template parents are local indices. The root keeps the caller's name/parent;
// all other component values (including local transform/active) use the template.
inline bool instantiate_template(EntityHandle root,const Entity* prototypes,size_t count,size_t root_index,
                                 const TemplateBinding* bindings,size_t binding_count,
                                 bool(*configure_components)(const EntityHandle*,size_t)=nullptr) {
    if(!root.get()||!prototypes||!count||count>32||root_index>=count||binding_count>count||(binding_count&&!bindings))return false;
    auto* root_binding=find_binding(root);
    if(!root_binding||root_binding->started||prototypes[root_index].parent!=-1)return false;
    int binding_index[32];for(auto& index:binding_index)index=-1;
    for(size_t i=0;i<binding_count;++i){
        const auto& binding=bindings[i];
        if(binding.entity>=count||binding_index[binding.entity]>=0)return false;
        const auto* type=find_class(binding.class_id);
        if(!type||!type->create||!type->release)return false;
        if(binding.entity==root_index&&binding.class_id!=root_binding->type->id)return false;
        binding_index[binding.entity]=int(i);
    }
    for(size_t i=0;i<count;++i){
        if(i!=root_index&&prototypes[i].parent<0)return false;
        int parent=int(i);
        for(size_t depth=0;parent>=0;++depth){if(size_t(parent)>=count||depth>=count)return false;parent=prototypes[parent].parent;}
    }
    EntityHandle owners[32]={};owners[root_index]=root;
    auto rollback=[&](){for(size_t i=0;i<count;++i)if(i!=root_index&&owners[i].get())destroy_entity(owners[i].get());};
    // Complete all reservations before modifying hierarchy or executing typed
    // initializers. Failure returns all capacity without unstarted destroy hooks.
    for(size_t i=0;i<count;++i)if(i!=root_index){
        if(binding_index[i]>=0)owners[i]=reserve(bindings[binding_index[i]].class_id,prototypes[i].name,nullptr,root);
        else owners[i]=handle(create_entity(prototypes[i].name));
        if(!owners[i].get()){rollback();return false;}
    }
    const int root_parent=root.get()->parent;
    char root_name[129];for(size_t i=0;i<129;++i)root_name[i]=root.get()->name[i];
    for(size_t i=0;i<count;++i){
        auto* entity=owners[i].get();const auto generation=entity->generation;
        *entity=prototypes[i];entity->generation=generation;entity->alive=true;
        entity->parent=i==root_index?root_parent:int(owners[size_t(prototypes[i].parent)].index);
        if(i==root_index)entity->set_name(root_name);
    }
    for(size_t i=0;i<binding_count;++i){
        const auto& binding=bindings[i];auto* instance=find_binding(owners[binding.entity]);
        if(!instance){rollback();return false;}
        if(binding.apply)binding.apply(instance->binding.behaviour,owners,count);
        if(!root.get()){rollback();return false;}
    }
    // Optional bounded component registration shares the same reservations and
    // rollback, before any start callback. It never allocates extra entities.
    if(configure_components&&!configure_components(owners,count)){rollback();return false;}
    return true;
}
}
