#include "actor_scene_fixture.hpp"
#include "../../resources/templates/dungeon-complete/assets/scripts/DungeonCombat/DungeonEncounter.hpp"
#include <cstdio>
#include <cstring>
using namespace epok;

// Independent integer transcription of the resident game's scheduling/attack
// rules. Rendering, collision results and movement remain caller inputs.
struct OriginalGuard {
    int32_t timer=4506,windup=0,flash=0,dead=0,sight=137,ai=0;
    bool patrol=false,visible=false;
    void start(unsigned n){*this={};sight=int32_t(n%3+1)*137;ai=(n&1)?-68:0;}
    unsigned schedule(int32_t dt,int32_t& elapsed,bool& query){
        elapsed=0;query=false;
        if(dead>0){dead-=dt;return dead<=0?6:0;}
        ai+=dt;if(ai<137)return 0;
        elapsed=ai;ai=0;timer-=elapsed;if(flash>0)flash-=elapsed;sight-=elapsed;
        query=sight<=0;return 7;
    }
    unsigned decide(int32_t dt,bool close,bool observed,bool within,bool goal){
        if(!close)visible=false;
        else if(sight<=0){visible=observed;sight=410;}
        if(close&&visible){
            if(windup>0){windup-=dt;if(windup<=0){timer=8602;return 5;}return 0;}
            if(timer<=0){windup=2048;return 4;}
            return within?1:2;
        }
        windup=0;if(goal)patrol=!patrol;return 3;
    }
};
struct OriginalBoss {
    int32_t timer=7373,windup=0,flash=0;
    unsigned advance(uint32_t health,int32_t dt,bool observed,bool within){
        const bool fan=health<=140;timer-=dt;if(flash>0)flash-=dt;
        if(windup>0){windup-=dt;if(windup<=0){timer=fan?5530:7782;return 5;}return 0;}
        if(timer<=0&&observed){windup=fan?2048:2867;return 4;}
        return within?1:2;
    }
};
static void same(DungeonGuardState s,const OriginalGuard& o){
    assert(s.timer.raw()==o.timer&&s.windup.raw()==o.windup&&s.flash.raw()==o.flash);
    assert(s.dead_time.raw()==o.dead&&s.sight_timer.raw()==o.sight&&s.ai_time.raw()==o.ai);
    assert(s.patrol==o.patrol&&s.can_see==o.visible);
}
static void same(DungeonBossState s,const OriginalBoss& o){assert(s.timer.raw()==o.timer&&s.windup.raw()==o.windup&&s.flash.raw()==o.flash);}
static unsigned shot_reference(){
    DungeonEncounter rules;uint32_t random=0x1836abcdu;unsigned checks=0;
    static_assert(sizeof(DungeonEncounterShotTuning)==9*4);
    for(unsigned sample=0;sample<2048;++sample){
        auto next=[&](){random=random*1664525u+1013904223u;return Fixed(int32_t(random%8193)-4096,Fixed::RAW);};
        const GameplayVector3 origin{next(),next(),next()},dir{next(),next(),next()};
        for(bool fan:{false,true}){
            const auto count=rules.pellet_count(fan);assert(count==(fan?3u:1u));
            for(uint32_t ordinal=0;ordinal<count;++ordinal){
                const int32_t expected=fan?int32_t(ordinal)-1:0;
                const auto pellet=rules.pellet_index(ordinal,fan);assert(pellet==expected);
                const auto actual=rules.projectile(origin,dir,pellet,fan);
                // Original firing loop: angle, then rotated direction, then speed.
                const Fixed angle(expected*12*4096,Fixed::RAW),c=cos_degrees(angle),s=sin_degrees(angle),speed=fan?Fixed(8.0):Fixed(7.0);
                const Fixed vx=dir.x*c+dir.z*s,vz=dir.z*c-dir.x*s;
                assert(actual.origin.x==origin.x&&actual.origin.y==origin.y&&actual.origin.z==origin.z);
                assert(actual.velocity.x==vx*speed&&actual.velocity.y==dir.y*speed&&actual.velocity.z==vz*speed);
                assert(actual.lifetime==2.5&&actual.tag==1&&rules.shot_size(fan)==(fan?Fixed(.32):Fixed(.24)));++checks;
            }
        }
        const auto target=rules.shot_target(origin);assert(target.x==origin.x&&target.y==origin.y+Fixed(.95)&&target.z==origin.z);++checks;
    }
    assert(rules.shot_damage(false)==12&&rules.shot_damage(true)==16);
    assert(rules.pellet_index(UINT32_MAX,true)==0&&rules.pellet_index(0,false)==0);
    rules.shot_tuning={9.0,11.0,3.0,30.0,.5,.75,24,32,1.5};
    for(bool fan:{false,true})for(uint32_t n=0;n<rules.pellet_count(fan);++n){
        const auto pellet=rules.pellet_index(n,fan),expected=fan?int32_t(n)-1:0;
        const auto shot=rules.projectile({1.0,2.0,3.0},{1.0,.5,0.0},pellet,fan);
        const Fixed angle(expected*30*4096,Fixed::RAW),speed=fan?Fixed(11.0):Fixed(9.0);
        assert(shot.velocity.x==cos_degrees(angle)*speed&&shot.velocity.y==Fixed(.5)*speed&&shot.velocity.z==-sin_degrees(angle)*speed);
        assert(shot.lifetime==3.0&&rules.shot_size(fan)==(fan?Fixed(.75):Fixed(.5)));++checks;
    }
    assert(rules.shot_target({1.0,2.0,3.0}).y==3.5&&rules.shot_damage(false)==24&&rules.shot_damage(true)==32);
    rules.shot_tuning.damage=UINT32_MAX;assert(rules.shot_damage(false)==UINT32_MAX);
    // Wrapped authored angles and all signed lane inputs stay finite in Q12.
    for(auto angle:{Fixed(-12.0),Fixed(360.0),Fixed(INT32_MIN,Fixed::RAW),Fixed(INT32_MAX,Fixed::RAW)}){
        rules.shot_tuning.fan_degrees=angle;
        for(int32_t pellet:{INT32_MIN,-1,0,1,INT32_MAX}){
            const auto shot=rules.projectile({}, {1.0,0.0,0.0},pellet,true);
            assert(shot.velocity.x>=Fixed(-11.0)&&shot.velocity.x<=Fixed(11.0));
            assert(shot.velocity.z>=Fixed(-11.0)&&shot.velocity.z<=Fixed(11.0));++checks;
        }
    }
    std::printf("PASS %u exact legacy/configured enemy-shot checkpoints; fan order, damage, sizes, target height and angle bounds\n",checks);
    return checks;
}
int main(int argc,char** argv){
    shot_reference();if(argc>1&&std::strcmp(argv[1],"--shots-only")==0)return 0;
    DungeonEncounter rules;
    assert(rules.guard_timing.ai_interval.raw()==137&&rules.guard_timing.guard_initial_delay.raw()==4506);
    assert(rules.guard_timing.guard_cooldown.raw()==8602&&rules.guard_timing.guard_flash.raw()==573&&rules.guard_timing.guard_death.raw()==3482);
    assert(rules.boss_timing.boss_initial_delay.raw()==7373&&rules.boss_timing.boss_cooldown.raw()==7782&&rules.boss_timing.boss_fan_cooldown.raw()==5530);
    assert(rules.boss_timing.boss_windup.raw()==2867&&rules.boss_timing.boss_flash.raw()==655);
    unsigned checks=0,guard_fires=0,boss_fires=0,charges=0,unloads=0;
    for(unsigned n=0;n<6;++n)for(int32_t raw:{68,69,137}){
        OriginalGuard old;old.start(n);auto state=rules.guard_start(n);auto health=rules.starting_health(false);same(state,old);
        for(unsigned frame=0;frame<7200;++frame){
            if(frame%1800==0){old.start(n);state=rules.guard_start(n);health=rules.starting_health(false);}
            const Fixed dt(raw,Fixed::RAW);int32_t elapsed;bool query;
            const auto result=rules.guard_schedule(state,dt);const unsigned expected=old.schedule(raw,elapsed,query);
            assert(result.action==expected&&result.dt.raw()==elapsed);
            if(expected==7)assert((result.state.sight_timer<=0.0)==query);
            state=result.state;same(state,old);++checks;
            if(result.action==6){++unloads;old.start(n);state=rules.guard_start(n);continue;}
            if(result.action!=7)continue;
            const bool close=frame%600<480,observed=frame%700<560,within=frame%240<120,goal=frame%47==0;
            const auto step=rules.guard_decide(state,result.dt,close,observed,within,goal);
            const auto action=old.decide(elapsed,close,observed,within,goal);
            assert(step.action==action&&step.dt.raw()==elapsed);state=step.state;same(state,old);++checks;
            guard_fires+=action==5;charges+=action==4;
            if(frame%173==0&&health.current>0){
                const uint32_t before=health.current;const auto change=HealthLibrary::damage(health,20);
                health=change.state;state=rules.guard_hit(state,change.depleted);old.flash=573;
                assert(health.current==(before>20?before-20:0));if(before<=20)old.dead=3482;
                same(state,old);++checks;
            }
        }
    }
    for(int32_t raw:{68,69,137,4096}){
        OriginalBoss old;auto state=rules.boss_start();auto health=rules.starting_health(true);same(state,old);
        for(unsigned frame=0;frame<9600;++frame){
            if(frame%2400==0){old={};state=rules.boss_start();health=rules.starting_health(true);}
            const bool observed=frame%700<530,within=frame%240<120;const Fixed dt(raw,Fixed::RAW);
            assert(rules.boss_needs_sight(state,dt)==(old.windup<=0&&old.timer-raw<=0));
            const auto step=rules.boss_advance(state,health.current,dt,observed,within);
            const auto action=old.advance(health.current,raw,observed,within);state=step.state;same(state,old);
            assert(step.action==action&&step.fan==(health.current<=140));
            assert(step.speed==(step.fan?Fixed(1.3):Fixed(.85)));boss_fires+=action==5;charges+=action==4;++checks;
            if(frame%137==0&&health.current>0){
                const auto before=health.current;const auto change=HealthLibrary::damage(health,20);
                health=change.state;state=rules.boss_hit(state);old.flash=655;same(state,old);
                assert(health.current==(before>20?before-20:0));++checks;
            }
        }
    }
    for(bool defeated:{false,true})for(bool explore:{false,true})for(bool near:{false,true}){
        assert(rules.should_activate(defeated,explore,near)==(!defeated&&explore&&near));
        auto state=rules.guard_start(0);assert(rules.should_unload(state,!explore,near)==(!explore||!near));
        state.dead_time=1.0;assert(!rules.should_unload(state,!explore,near));
    }
    rules.health_tuning.guard_health=75;rules.health_tuning.boss_health=360;rules.health_tuning.boss_phase_health=200;
    rules.guard_timing.guard_windup=.25;rules.boss_timing.boss_fan_speed=2.0;
    assert(rules.starting_health(false).current==75&&rules.starting_health(true).current==360);
    auto state=rules.guard_start(0);state.timer=0.0;state.sight_timer=0.0;
    assert(rules.guard_decide(state,Fixed(.1),true,true,false,false).state.windup==.25);
    assert(rules.boss_advance(rules.boss_start(),190,.1,false,false).speed==2.0);
    for(auto dt:{Fixed(0.0),Fixed(-1.0),Fixed(1.1)}){
        const auto guard=rules.guard_schedule(state,dt);assert(!guard.action&&guard.dt==0.0&&guard.state.timer==state.timer);
        const auto boss=rules.boss_advance(rules.boss_start(),280,dt,true,true);assert(!boss.action&&boss.state.timer==rules.boss_timing.boss_initial_delay);
    }
    assert(guard_fires>0&&boss_fires>0&&charges>0&&unloads>0);
    std::printf("PASS %u exact guard/boss policy checkpoints; %u/%u fires, %u charges, %u death unloads; editable rules and timing boundaries\n",checks,guard_fires,boss_fires,charges,unloads);
}
