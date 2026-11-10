#include "actor_scene_fixture.hpp"
#include "../../resources/templates/dungeon-complete/assets/scripts/DungeonCombat/DungeonPlayerVitals.hpp"
#include <cstdio>
using namespace epok;

// Independent reference for the original game's int health and Q12 cooldown.
struct OriginalPlayerVitals {
    int health=100;int32_t immunity=0;
    void advance(int32_t dt){if(immunity>0){immunity-=dt;if(immunity<0)immunity=0;}}
    void damage(int amount,bool enabled){if(!enabled||amount<=0||immunity>0)return;health-=amount;if(health<0)health=0;immunity=2867;}
    void heal(int amount){health+=amount;if(health>100)health=100;}
};
static void same(const DungeonPlayerVitals& rules,const OriginalPlayerVitals& original){
    assert(rules.valid&&rules.health.maximum==100&&rules.health.current==uint32_t(original.health));
    assert(rules.immunity.raw()==original.immunity&&rules.can_heal()==(original.health<100));
}
int main(){
    static_assert(sizeof(DungeonVitalsTuning)==8);
    assert(Fixed(.7).raw()==2867);
    unsigned checks=0;
    for(int raw:{68,69,137,4096}){
        DungeonPlayerVitals rules;OriginalPlayerVitals old;rules.start();same(rules,old);
        for(unsigned frame=0;frame<9600;++frame){
            if(frame%600==0){rules.start();old={};}
            rules.advance(Fixed(raw,Fixed::RAW));old.advance(raw);same(rules,old);++checks;
            // The game's run-flow disables damage after depletion. An enabled
            // hit on already-empty health is covered separately as a no-op.
            if(frame%17==0){const int amount=frame%51==0?16:12;const bool enabled=frame%113>9&&old.health>0;
                const auto change=rules.damage(amount,enabled);old.damage(amount,enabled);same(rules,old);
                assert(change.depleted==(change.applied>0&&rules.health.current==0));++checks;}
            if(frame%71==0){rules.heal(35);old.heal(35);same(rules,old);++checks;}
        }
    }
    DungeonPlayerVitals rules;rules.start();auto change=rules.damage(12,true);
    assert(change.applied==12&&rules.health.current==88&&rules.immunity.raw()==2867);
    rules.advance(Fixed(2866,Fixed::RAW));assert(rules.immunity.raw()==1);
    assert(!rules.damage(20,true).applied);rules.advance(Fixed(1,Fixed::RAW));
    assert(rules.damage(20,true).applied==20);
    for(auto dt:{Fixed(0.0),Fixed(-1.0),Fixed(1.1)}){rules.advance(dt);assert(rules.immunity.raw()==2867);}
    rules.start();assert(!rules.damage(20,false).applied&&!rules.damage(-20,true).applied);
    rules.tuning={UINT32_MAX,0.0};rules.start();
    assert(rules.valid&&rules.damage(INT32_MAX,true).state.current==0x80000000u);
    assert(rules.damage(INT32_MAX,true).state.current==1);
    assert(rules.damage(1,true).depleted&&!rules.damage(1,true).depleted);
    assert(rules.heal(INT32_MAX).revived&&rules.heal(INT32_MAX).state.current==0xfffffffeu);
    assert(rules.heal(35).state.current==UINT32_MAX&&!rules.can_heal());
    rules.assign({100,101},0.0,true);
    assert(!rules.can_heal()&&rules.damage(20,true).result==HealthResult::InvalidState);
    assert(rules.heal(35).result==HealthResult::InvalidState&&rules.health.current==101);
    for(auto tuning:{DungeonVitalsTuning{0,.7},DungeonVitalsTuning{100,-.1},DungeonVitalsTuning{100,60.1}}){
        rules.tuning=tuning;rules.start();assert(!rules.valid&&!rules.can_heal());
        assert(!rules.damage(20,true).applied&&!rules.heal(35).applied);
    }
    rules.tuning={280,60.0};rules.start();assert(rules.valid);
    rules.damage(20,true);assert(rules.immunity==60.0);
    std::printf("PASS %u exact original-game health/cooldown checkpoints; immunity boundary, editable tuning, invalid inputs and uint32 limits\n",checks);
}
