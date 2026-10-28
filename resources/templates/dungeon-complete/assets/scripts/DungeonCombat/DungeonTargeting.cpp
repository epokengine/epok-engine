#include "DungeonTargeting.hpp"
#include "scene.hh"

namespace third_person_targeting {
namespace {
bool starts_with(const char* value,const char* prefix){
    while(*prefix&&*value==*prefix){++value;++prefix;}
    return !*prefix;
}
}
bool candidate(const epok::ActorData* value){
    return value&&epok::is_active(value)&&value->mesh&&value->collider.enabled&&starts_with(value->name,"Target ");
}
int64_t horizontal_distance_squared(const epok::Transform& from,const epok::ActorData& target){
    const int64_t x=(target.transform.position[0]-from.position[0]).raw();
    const int64_t z=(target.transform.position[2]-from.position[2]).raw();
    return x*x+z*z;
}
bool nearby(const epok::Transform& from,const epok::ActorData& target,epok::Fixed range){
    const int64_t raw=range.raw();
    return horizontal_distance_squared(from,target)<=raw*raw;
}
bool visible(const epok::Transform& from,const epok::ActorData& target,const epok::SpatialQueryBatch* queries){
    const epok::Fixed origin[3]={from.position[0],from.position[1]+epok::Fixed(1.1),from.position[2]};
    epok::Fixed delta[3];for(int k=0;k<3;++k)delta[k]=target.transform.position[k]-origin[k];
    delta[1]+=target.collider.center[1]*target.transform.scale[1];
    return !(queries?queries->raycast(origin,delta,1):epok::raycast(origin,delta,1));
}
epok::DataHandle nearest(const epok::Transform& from,epok::Fixed range,const epok::SpatialQueryBatch* queries){
    epok::ActorData* best=nullptr;int64_t best_distance=0;
    for(size_t index=0;index<epok::object_count;++index){
        auto& value=epok::objects[index];
        if(!candidate(&value)||!nearby(from,value,range)||!visible(from,value,queries))continue;
        const int64_t distance=horizontal_distance_squared(from,value);
        if(!best||distance<best_distance){best=&value;best_distance=distance;}
    }
    return best?epok::handle(best):epok::DataHandle{};
}
epok::DataHandle next(const epok::Transform& from,const epok::DataHandle& selected,epok::Fixed range,const epok::SpatialQueryBatch* queries){
    const auto* current=selected.get();
    if(!candidate(current)||!nearby(from,*current,range))return nearest(from,range,queries);
    const int64_t current_distance=horizontal_distance_squared(from,*current);
    epok::ActorData* best=nullptr;int64_t best_distance=0;size_t best_index=0;
    for(size_t index=0;index<epok::object_count;++index){
        auto& value=epok::objects[index];
        if(&value==current||!candidate(&value)||!nearby(from,value,range)||!visible(from,value,queries))continue;
        const int64_t distance=horizontal_distance_squared(from,value);
        if(distance<current_distance||(distance==current_distance&&index<=selected.index))continue;
        if(!best||distance<best_distance||(distance==best_distance&&index<best_index)){
            best=&value;best_distance=distance;best_index=index;
        }
    }
    return best?epok::handle(best):epok::DataHandle{};
}
}
