#include "actor_scene_fixture.hpp"
#include "../../resources/templates/dungeon-complete/assets/scripts/DungeonCombat/DungeonPlayerCombat.hpp"
#include <cstdio>
using namespace epok;

// Exact independent translation of the legacy controller's ordered attack code.
static DungeonAttackStep original(DungeonAttackState state,bool grounded,bool jumped,bool held,bool playing,bool ready){
    DungeonAttackStep out;out.state=state;
    auto& s=out.state;
    if(s.attacking&&(jumped||!grounded)){s.attacking=false;out.invalidate_locomotion=true;}
    if(s.attacking&&!playing){
        if(s.phase==0){s.phase=1;out.clip=1;s.shot_sent=false;}
        else if(s.phase==1){s.phase=held?0:2;out.clip=int32_t(s.phase);}
        else{s.attacking=false;out.invalidate_locomotion=true;}
    }
    if(!s.attacking&&grounded&&!jumped&&held&&ready){s.phase=0;s.attacking=true;s.shot_sent=false;out.clip=0;out.starting=true;}
    if(!s.attacking)return out;
    if(!s.shot_sent&&s.phase==1){s.shot_sent=true;out.fire=true;}
    return out;
}
static void same(const DungeonAttackStep& a,const DungeonAttackStep& b){
    assert(a.state.phase==b.state.phase&&a.state.attacking==b.state.attacking&&a.state.shot_sent==b.state.shot_sent);
    assert(a.clip==b.clip&&a.fire==b.fire&&a.invalidate_locomotion==b.invalidate_locomotion&&a.starting==b.starting);
}
int main(){
    DungeonPlayerCombat rules;unsigned checks=0;
    for(unsigned phase=0;phase<5;++phase)for(unsigned state=0;state<4;++state)for(unsigned input=0;input<32;++input){
        const DungeonAttackState initial{phase,bool(state&1),bool(state&2)};
        same(rules.attack(initial,input&1,input&2,input&4,input&8,input&16),original(initial,input&1,input&2,input&4,input&8,input&16));++checks;
        assert(rules.blocks_movement(initial,input&1,input&4)==(initial.attacking||bool((input&1)&&(input&4))));
    }
    DungeonAttackState state{},reference{};
    uint32_t emitted=0;
    for(unsigned tick=0;tick<24000;++tick){
        const bool grounded=tick%109<100,jumped=tick%97==0,held=tick%211<170,playing=tick%17!=0,ready=tick%313!=0;
        auto next=rules.attack(state,grounded,jumped,held,playing,ready);
        auto expected=original(reference,grounded,jumped,held,playing,ready);same(next,expected);++checks;
        // Initial playback failure is acknowledged by the caller; no shot fired.
        if(next.starting&&tick%5==0){next.state.attacking=false;expected.state.attacking=false;assert(!next.fire);}
        state=next.state;reference=expected.state;if(next.fire)++emitted;
    }
    assert(emitted>100);
    for(unsigned mask=0;mask<8;++mask){const auto action=rules.target_input(mask&1,mask&2,mask&4);assert(action==((mask&2)?1:(!(mask&1)&&(mask&4)?2:0)));++checks;}
    uint8_t legacy_phase=0;
    for(unsigned tick=0;tick<12000;++tick){
        const bool valid=tick%97<90,near=tick%83<75;bool query=false;
        const auto actual=rules.target_check(legacy_phase,valid,near);
        if(valid&&near&&++legacy_phase>=8){legacy_phase=0;query=true;}
        assert(actual.phase==legacy_phase&&actual.keep==(valid&&near)&&actual.query_sight==query);++checks;
    }
    for(unsigned mask=0;mask<16;++mask)assert(rules.candidate(mask&1,mask&2,mask&4,mask&8)==(mask==15));
    const uint64_t distances[]={0,1,100,UINT32_MAX,0x100000000ull,0x100000001ull,0x7fffffffffffffffull};
    for(uint64_t distance:distances)for(uint64_t selected:distances)for(uint32_t index=0;index<8;++index){
        const auto a=rules.make_target_distance(uint32_t(distance>>32),uint32_t(distance));
        const auto b=rules.make_target_distance(uint32_t(selected>>32),uint32_t(selected));
        assert(rules.after_selected(a,index,b,3)==!(distance<selected||(distance==selected&&index<=3)));
        for(bool cycling:{false,true})assert(rules.prefer_candidate(a,index,true,b,3,cycling)==(distance<selected||(cycling&&distance==selected&&index<3)));
        assert(rules.prefer_candidate(a,index,false,b,3,false));++checks;
    }
    const auto shot=rules.projectile({1.0,2.0,3.0},{.5,-.25,1.0});
    assert(shot.origin.y==2.0&&shot.velocity.x==11.0&&shot.velocity.y==-5.5&&shot.velocity.z==22.0&&shot.lifetime==Fixed(1.7)&&shot.tag==0);
    rules.tuning.projectile_speed=9.0;rules.tuning.projectile_lifetime=3.0;rules.tuning.visibility_period=3;
    const auto edited=rules.projectile({}, {1.0,0.0,0.0});assert(edited.velocity.x==9.0&&edited.lifetime==3.0);
    const auto check=rules.target_check(2,true,true);assert(check.phase==0&&check.query_sight);
    std::printf("PASS %u exact legacy attack/target checkpoints; playback failures, target ties, input precedence and editable tuning\n",checks);
}
