#include "DungeonTargeting.hpp"
#include "scene.hh"

namespace third_person_targeting {
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
}
