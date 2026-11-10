#include "actor_scene_fixture.hpp"
#include <cstdio>
#include <initializer_list>
using namespace epok;

static uint32_t cases=0;
static void compare(uint32_t maximum,uint32_t current,int32_t amount){
    const HealthState original{maximum,current};
    const auto damage=HealthLibrary::damage(original,amount);
    const auto heal=HealthLibrary::heal(original,amount);
    const uint64_t positive=amount>0?uint32_t(amount):0;
    const uint32_t taken=uint32_t(positive<current?positive:current);
    const uint32_t restored=uint32_t(positive<uint64_t(maximum)-current?positive:uint64_t(maximum)-current);
    assert(original.maximum==maximum&&original.current==current);
    assert(damage.state.maximum==maximum&&damage.state.current==current-taken&&damage.applied==taken);
    assert(heal.state.maximum==maximum&&heal.state.current==current+restored&&heal.applied==restored);
    assert(damage.depleted==(taken>0&&taken==current)&&!damage.revived);
    assert(heal.revived==(restored>0&&current==0)&&!heal.depleted);
    assert(damage.result==(taken?HealthResult::Applied:HealthResult::Unchanged));
    assert(heal.result==(restored?HealthResult::Applied:HealthResult::Unchanged));
    ++cases;
}
int main(){
    static_assert(sizeof(HealthState)==8&&sizeof(HealthChange)==16);
    for(uint32_t maximum:{60u,100u,280u})
        for(uint32_t current=0;current<=maximum;++current)
            for(int32_t amount=-2;amount<=301;++amount)compare(maximum,current,amount);
    for(uint32_t maximum:{0u,1u,0x7fffffffu,0x80000000u,0xfffffffeu,0xffffffffu})
        for(uint32_t current:{0u,maximum/2,maximum})
            for(int32_t amount:{INT32_MIN,-1,0,1,20,35,INT32_MAX})compare(maximum,current,amount);
    const auto state=HealthLibrary::make_state(100,UINT32_MAX);
    assert(state.maximum==100&&state.current==100);
    auto change=HealthLibrary::damage(state,12);
    assert(change.state.current==88&&change.applied==12);
    change=HealthLibrary::heal(change.state,35);
    assert(change.state.current==100&&change.applied==12);
    change=HealthLibrary::damage(change.state,INT32_MAX);
    assert(change.depleted&&change.state.current==0);
    change=HealthLibrary::damage(change.state,20);
    assert(!change.depleted&&change.result==HealthResult::Unchanged);
    change=HealthLibrary::heal(change.state,1);
    assert(change.revived&&change.state.current==1);
    const HealthState invalid{100,101};
    for(auto value:{HealthLibrary::damage(invalid,20),HealthLibrary::heal(invalid,35)}){
        assert(value.result==HealthResult::InvalidState&&value.state.maximum==100&&value.state.current==101);
        assert(!value.applied&&!value.depleted&&!value.revived);
    }
    assert(!HealthLibrary::valid(invalid)&&!HealthLibrary::is_depleted(invalid));
    assert(HealthLibrary::is_depleted(HealthLibrary::make_state(0,100)));
    std::printf("PASS %u independent health comparisons; limits, invalid states, depletion and revival\n",cases);
}
