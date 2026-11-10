#include "actor_scene_fixture.hpp"
#ifndef DRAFT_COMBAT_HEADER
#define DRAFT_COMBAT_HEADER "../../resources/templates/dungeon-complete/assets/scripts/DungeonCombat/DungeonPlayerCombat.hpp"
#endif
#include DRAFT_COMBAT_HEADER
#include <cstdio>
using namespace epok;
// The oracle methods below are literal extractions of the pre-change project.
// The fixture uses ordinary ActorData, not replacement math or SDK behavior.
struct LegacyRef {ActorData* ptr=nullptr;ActorData* get()const{return ptr;}};
struct LegacyGame {
    static constexpr uint32_t damage_flash_duration_us=220000;
    uint32_t damage_flash_us=0;
    LegacyRef damage_border{};
    struct Effect {LegacyRef data{};uint16_t age=0,kind=0;bool playing=false;};
    Effect effects[4]{};
    void update_damage_feedback(uint32_t microseconds);
    void effect(const Fixed*,uint16_t);
    void advance_effects();
};
void LegacyGame::update_damage_feedback(uint32_t microseconds){
    // Presentation follows rendered time, including death holds and menus.
    // Keep a visible frame at least one pixel wide until it is disabled: zero
    // borders intentionally mean a filled Image, not an empty one.
    damage_flash_us=microseconds>=damage_flash_us?0:damage_flash_us-microseconds;
    if(auto* d=damage_border.get()){
        d->image.enabled=damage_flash_us!=0;
        if(d->image.enabled){
            const uint16_t width=uint16_t((6*damage_flash_us+damage_flash_duration_us-1)/damage_flash_duration_us);
            for(auto& edge:d->image.borders)edge=width;
            d->image.color[0]=uint8_t(96+128*damage_flash_us/damage_flash_duration_us);
            d->image.color[1]=16;d->image.color[2]=20;
        }
    }
}

void LegacyGame::effect(const Fixed* point,uint16_t kind){
    for(auto& f:effects)if(!f.playing)if(auto* d=f.data.get()){
        f.playing=true;f.age=0;f.kind=kind;for(int k=0;k<3;++k)d->transform.position[k]=point[k];
        d->sprite.size[0]=d->sprite.size[1]=kind?Fixed(.48):Fixed(.28);d->sprite.enabled=true;
        const unsigned frame=kind?6:4;d->sprite.region[0]=(frame%8)*32;d->sprite.region[1]=(frame/8)*32;return;
    }
}

void LegacyGame::advance_effects(){    for(auto& f:effects)if(f.playing){auto* d=f.data.get();if(!d){f.playing=false;continue;}++f.age;
        const unsigned frame=f.kind?6+f.age/2:(f.age<4?4:5),end=f.kind?10:6;
        if(f.age>=end){d->sprite.enabled=false;f.playing=false;}else{d->sprite.region[0]=(frame%8)*32;d->sprite.region[1]=(frame/8)*32;}}}
static void legacy_shot_visual(ActorData* d){d->sprite.size[0]=d->sprite.size[1]=.18;d->sprite.color[0]=d->sprite.color[1]=d->sprite.color[2]=255;}
static uint32_t cap(uint32_t value,uint32_t low,uint32_t high){return value<low?low:value>high?high:value;}
static DungeonDamageFeedback wide_reference(const DungeonDamageFeedbackTuning& t,uint32_t left,uint32_t elapsed,bool restart){
    const uint64_t duration=cap(t.duration_us,0,10000000);if(!duration)return {};
    const uint64_t before=restart?duration:cap(left,0,uint32_t(duration));if(elapsed>=before)return {};
    const uint64_t remaining=before-elapsed,width=cap(t.border_pixels,1,64),red=cap(t.red_base,0,255),span=cap(t.red_span,0,255-uint32_t(red));
    return {uint32_t(remaining),true,uint32_t((width*remaining+duration-1)/duration),uint32_t(red+span*remaining/duration),cap(t.green,0,255),cap(t.blue,0,255)};
}
static void same(DungeonDamageFeedback a,DungeonDamageFeedback b){
    assert(a.remaining_us==b.remaining_us&&a.enabled==b.enabled&&a.border_pixels==b.border_pixels&&a.red==b.red&&a.green==b.green&&a.blue==b.blue);
}
int main(){
    static_assert(sizeof(DungeonImpactTrace)<=2*4&&sizeof(DungeonPlayerShotVisual)<=4*4&&sizeof(DungeonCombatEffectStep)<=4*4);
    static_assert(sizeof(DungeonDamageFeedback)<=6*4&&sizeof(DungeonDamageRumble)<=3*4);
    static_assert(sizeof(DungeonCombatEffectTuning)<=9*4&&sizeof(DungeonDamageFeedbackTuning)<=9*4);
    DungeonPlayerCombat rules;uint32_t checks=0;
    ActorData border{};LegacyGame game;game.damage_border.ptr=&border;
    for(uint32_t remaining=0;remaining<=220000;++remaining){
        game.damage_flash_us=remaining;game.update_damage_feedback(0);const auto actual=rules.damage_feedback(remaining,0,false);
        assert(actual.remaining_us==game.damage_flash_us&&actual.enabled==border.image.enabled);
        if(actual.enabled){for(auto edge:border.image.borders)assert(edge==actual.border_pixels);
            assert(actual.red==border.image.color[0]&&actual.green==border.image.color[1]&&actual.blue==border.image.color[2]);}
        ++checks;
    }
    const auto initial=rules.damage_feedback(0,0,true);assert(initial.remaining_us==220000&&initial.enabled&&initial.border_pixels==6&&initial.red==224&&initial.green==16&&initial.blue==20);++checks;
    for(uint32_t dt:{0u,1u,33333u,219999u,220000u,UINT32_MAX}){
        game.damage_flash_us=220000;game.update_damage_feedback(dt);const auto value=rules.damage_feedback(220000,dt,false);
        assert(value.remaining_us==game.damage_flash_us&&value.enabled==border.image.enabled);++checks;
    }
    const auto rumble=rules.damage_rumble();assert(rumble.large==160&&rumble.small&&rumble.duration_ms==120);++checks;
    ActorData shot{};legacy_shot_visual(&shot);const auto visual=rules.shot_visual();
    assert(visual.size==shot.sprite.size[0]&&visual.red==shot.sprite.color[0]&&visual.green==shot.sprite.color[1]&&visual.blue==shot.sprite.color[2]);++checks;
    for(uint16_t kind:{uint16_t(0),uint16_t(1),uint16_t(7)}){
        ActorData sprite{};LegacyGame old;old.effects[0].data.ptr=&sprite;const Fixed origin[]={1.0,2.0,3.0};old.effect(origin,kind);
        auto current=rules.effect_step(kind,0,false);
        assert(current.playing==old.effects[0].playing&&current.age==old.effects[0].age&&current.size==sprite.sprite.size[0]);
        assert((current.frame%8)*32==sprite.sprite.region[0]&&(current.frame/8)*32==sprite.sprite.region[1]);++checks;
        for(uint32_t n=0;old.effects[0].playing&&n<20;++n){
            current=rules.effect_step(kind,current.age,true);old.advance_effects();assert(current.playing==old.effects[0].playing&&current.age==old.effects[0].age);
            if(current.playing)assert((current.frame%8)*32==sprite.sprite.region[0]&&(current.frame/8)*32==sprite.sprite.region[1]);++checks;
        }
    }
    for(bool enemy:{false,true})for(bool exists:{false,true})for(bool player:{false,true})for(uint32_t layer:{0u,1u,2u,3u,4u,5u,0x80000000u,UINT32_MAX}){
        const uint32_t expected=enemy?(player?2u:3u):(exists&&(layer&2)?1u:0u);assert(rules.impact(enemy,exists,player,layer)==expected);++checks;
    }
    assert(rules.trace_settings(false).mask==3&&rules.trace_settings(false).ignore_player);
    assert(rules.trace_settings(true).mask==5&&!rules.trace_settings(true).ignore_player);checks+=2;
    rules.impact_tuning={0x80000001u,UINT32_MAX};assert(rules.trace_settings(false).mask==0x80000001u&&rules.trace_settings(true).mask==UINT32_MAX);++checks;
    rules.shot_style={.4,20,40,80};auto edited=rules.shot_visual();assert(edited.size==Fixed(.4)&&edited.red==20&&edited.green==40&&edited.blue==80);++checks;
    rules.shot_style={-1.0,UINT32_MAX,256,0};edited=rules.shot_visual();assert(edited.size==0.0&&edited.red==255&&edited.green==255&&edited.blue==0);++checks;
    rules.shot_style.size=Fixed(INT32_MAX,Fixed::RAW);assert(rules.shot_visual().size==32.0);++checks;
    rules.effect_tuning={.6,.9,8,9,2,4,12,3,7};
    assert(rules.effect_step(0,0,false).frame==8&&rules.effect_step(0,2,false).frame==9&&rules.effect_step(0,4,false).playing==false);
    assert(rules.effect_step(1,3,false).frame==13&&rules.effect_step(1,7,false).playing==false&&rules.effect_step(1,0,false).size==Fixed(.9));checks+=2;
    rules.effect_tuning={-1.0,100.0,UINT32_MAX,UINT32_MAX,UINT32_MAX,UINT32_MAX,UINT32_MAX,0,UINT32_MAX};
    for(uint32_t age:{0u,1u,65534u,65535u,UINT32_MAX})for(bool advance:{false,true})for(uint32_t kind:{0u,1u,UINT32_MAX}){
        const auto fx=rules.effect_step(kind,age,advance);assert(fx.age<=65535&&fx.frame<=2047&&fx.size>=0.0&&fx.size<=32.0);
        const uint64_t bounded=age>65535?65535:age;assert(fx.age==bounded+(advance&&bounded<65535?1:0));++checks;
    }
    rules.effect_tuning.hit_end_age=0;assert(!rules.effect_step(1,0,false).playing);++checks;
    const uint32_t extremes[]={0,1,6,255,256,220000,10000000,UINT32_MAX};
    for(uint32_t duration:extremes)for(uint32_t width:extremes)for(uint32_t color:extremes){
        rules.feedback_tuning={duration,width,color,UINT32_MAX,UINT32_MAX,color,UINT32_MAX,UINT32_MAX,false};
        for(uint32_t remaining:{0u,1u,219999u,UINT32_MAX})for(uint32_t elapsed:{0u,1u,UINT32_MAX})for(bool restart:{false,true}){
            const auto value=rules.damage_feedback(remaining,elapsed,restart);same(value,wide_reference(rules.feedback_tuning,remaining,elapsed,restart));
            assert(value.border_pixels<=64&&value.red<=255&&value.green<=255&&value.blue<=255);
            if(value.enabled)assert(value.border_pixels>=1);++checks;
        }
        const auto output=rules.damage_rumble();assert(output.large<=255&&!output.small&&output.duration_ms<=60000);++checks;
    }
    rules.feedback_tuning={1000,9,20,40,30,50,60,70,false};
    const auto changed=rules.damage_feedback(0,250,true);assert(changed.remaining_us==750&&changed.border_pixels==7&&changed.red==50&&changed.green==30&&changed.blue==50);++checks;
    std::printf("PASS %u feedback/impact/FX checkpoints; literal pre-change oracle, configured values, full UInt32 bounds\n",checks);
}
