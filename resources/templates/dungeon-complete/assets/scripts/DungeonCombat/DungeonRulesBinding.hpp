#pragma once
#include "epok.hpp"

// Cache type validation only. Every access checks the current registry, handle
// generation and live storage, including deferred destruction and slot reuse.
template<class Rules> class DungeonRulesBinding {
    epok::ObjectRegistry* registry_=nullptr;
    epok::ObjectId identity_{};
    Rules* value_=nullptr;
public:
    Rules* resolve(epok::ObjectId identity){
        auto* registry=epok::active_object_registry;
        auto* object=registry?registry->get(identity):nullptr;
        if(registry!=registry_||identity!=identity_||object!=value_){
            registry_=registry;identity_=identity;
            value_=registry?registry->resolve<Rules>(identity):nullptr;
        }
        return value_;
    }
};
