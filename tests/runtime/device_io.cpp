#define EPOK_WORLD_GAMEPLAY 1
#define EPOK_TEST_EXTRA_CLASSES \
 {PocketTransactionComponent::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::None,7,0,&object_construct<PocketTransactionComponent>,&object_destruct,sizeof(PocketTransactionComponent),alignof(PocketTransactionComponent),&ObjectPool<PocketTransactionComponent,2>::acquire,&ObjectPool<PocketTransactionComponent,2>::release},
#include "actor_scene_fixture.hpp"
#include <cstdio>
using namespace epok;

static void answer(uint32_t bytes=5){
    auto& box=epok_pocket_mailbox;box.received[0]=uint16_t(bytes);
    box.reply[0]=0xff;box.reply[2]=2;box.answered=box.request;
}
int main(){
    test_reset_scene();auto& box=epok_pocket_mailbox;box.magic=PocketMailbox::magic_value;
    auto* owner=test_registry.resolve<Actor3D>(test_owner(0));
    auto* other=test_registry.resolve<Actor3D>(test_owner(1));
    auto* link=test_level.add_component<PocketTransactionComponent>(*owner,"Device link");assert(link);
    auto* second=test_level.add_component<PocketTransactionComponent>(*other,"Other link");assert(second);
    assert(link->clear_request()&&link->set_request_byte(0,0x81)&&link->set_request_byte(1,0x58));
    assert(!link->set_request_byte(140,1)&&!link->set_request_byte(0,256));
    assert(!link->begin(0,-1).request&&link->last_rejection()==PocketError::BadArgument);
    assert(!link->begin(141,-1).request&&!link->begin(5,-2).request&&!link->begin(5,5).request);
    link->port=2;assert(!link->begin(5,-1).request&&link->last_rejection()==PocketError::BadPort);link->port=1;
    const auto first=link->begin(5,-1);assert(first.request&&first.component==link->id());
    assert(link->snapshot(first).valid&&link->snapshot(first).busy&&!link->snapshot(first).ready);
    assert(box.port==1&&box.send[0]==0x81&&box.send[1]==0x58);
    assert(!link->clear_request()&&!link->set_request_byte(1,0));
    assert(!link->begin(5,-1).request&&link->last_rejection()==PocketError::Busy);
    assert(!second->snapshot(first).valid&&!second->poll(first,1)&&!second->cancel(first));
    assert(!link->poll(first,1));answer();assert(link->poll(first,1));
    auto status=link->snapshot(first);assert(status.valid&&status.ready&&!status.busy&&status.received==5&&status.error==PocketError::OK);
    assert(link->reply_byte(first,2)==2&&link->reply_byte(first,5)==0);
    const auto next=link->begin(5,-1);assert(next.request!=first.request&&!link->snapshot(first).valid);
    // Cancel invalidates the handle; the shared bridge remains occupied until
    // its old response arrives, so a new client cannot steal an in-flight reply.
    assert(link->cancel(next)&&!link->snapshot(next).valid);
    assert(!second->begin(5,-1).request&&second->last_rejection()==PocketError::Refused);
    answer();const auto theirs=second->begin(5,-1);assert(theirs.request);answer();assert(second->poll(theirs,1));
    // Arbitrarily large elapsed values cannot wrap the timeout counter.
    const auto timeout=link->begin(5,-1);assert(timeout.request&&!link->poll(timeout,1));
    assert(link->poll(timeout,UINT32_MAX)&&link->snapshot(timeout).error==PocketError::Timeout);
    assert(!link->begin(5,-1).request&&!link->snapshot(timeout).valid);answer();
    // Automatic polling uses the same checked request and bounded time kernel.
    link->automatic_poll=true;const auto automatic=link->begin(5,-1);assert(automatic.request);answer();
    link->tick(Fixed(68,Fixed::RAW));assert(link->snapshot(automatic).ready);
    const auto disabled=link->begin(5,-1);assert(disabled.request);test_set_active(0,false);
    assert(!link->snapshot(disabled).valid);answer();test_set_active(0,true);
    const auto enabled=link->begin(5,-1);assert(enabled.request&&enabled.request!=disabled.request);answer();assert(link->poll(enabled,1));
    const auto ending=link->begin(5,-1);assert(ending.request);
    const auto dead_id=link->id();test_invalidate(0);assert(!test_registry.get(dead_id));answer();
    assert(second->clear_request());const auto survived=second->begin(5,-1);assert(survived.request);answer(1);
    assert(second->poll(survived,1)&&second->snapshot(survived).error==PocketError::NoCard);
    puts("Owned device transactions: packet bounds, checked handles, async replies, cancellation, timeout and lifecycle passed");
}
