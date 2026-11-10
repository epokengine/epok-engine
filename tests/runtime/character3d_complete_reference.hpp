#pragma once
// Frozen project integration shared by kernel and reflected-component oracles.
namespace complete_reference {
using namespace epok;
struct Cache {
    Fixed position[3]{},rotation[3]{};uint32_t revision=0;
    bool same(const ActorData& body,uint32_t current)const{
        bool equal=revision==current;
        for(unsigned i=0;i<3;++i)equal&=position[i]==body.transform.position[i]&&rotation[i]==body.transform.rotation[i];
        return equal;
    }
    void store(const ActorData& body,uint32_t current,const Fixed* prior_rotation){
        revision=current;for(unsigned i=0;i<3;++i){position[i]=body.transform.position[i];rotation[i]=prior_rotation[i];}
    }
};
// Frozen from DungeonPlayerController's original integration. The complete
// project's authored stair height is zero, so its lift branch never executes.
static bool original(ActorData& body,character3d::State& state,Cache& cache,GameplayVector2 desired,Fixed facing,bool jump,Fixed dt,uint32_t revision){
    Fixed response=(state.grounded?Fixed(12.0):Fixed(3.0))*dt;if(response>Fixed(1.0))response=1.0;
    state.velocity[0]+=(desired.x-state.velocity[0])*response;
    state.velocity[2]+=(desired.y-state.velocity[2])*response;
    const bool jumped=state.grounded&&jump;
    if(state.grounded)state.velocity[1]=jumped?Fixed(9.0):Fixed(0.0);
    else state.velocity[1]-=Fixed(24.0)*dt;
    const Fixed fall=state.grounded&&!jumped?Fixed(-0.05):state.velocity[1]*dt;
    const Fixed delta[3]={state.velocity[0]*dt,fall,state.velocity[2]*dt};
    const bool settled=state.grounded&&!jumped&&delta[0].raw()==0&&delta[2].raw()==0&&cache.same(body,revision);
    if(!settled){
        const auto result=move_and_slide(body,delta,0xffffffffu);state.grounded=result.grounded;
        if(state.velocity[1]>Fixed(0.0)&&result.displacement[1]+Fixed(0.001)<fall)state.velocity[1]=0.0;
    }
    cache.store(body,revision,body.transform.rotation);
    body.transform.rotation[1]=state.facing=facing;
    return jumped;
}
}
