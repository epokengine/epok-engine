#define EPOK_POCKET_SIO_HOST_TEST 1
#include "../../runtime/pocketstation_async.hpp"
#include <cassert>
#include <initializer_list>
static bool advance(epok::PocketRequest& request,unsigned lines=270){
    pocket_sio_test::counters[1].value=uint16_t(pocket_sio_test::counters[1].value+lines);
    return request.poll(16667);
}
int main(){
    using namespace epok;
    const uint8_t tx[]={0x81,0x58,0,0,0};
    for(unsigned port=0;port<2;++port){
        pocket_sio_test::reset(port);PocketRequest request;
        assert(request.begin(port,tx,sizeof(tx))&&request.busy()&&!request.ready());
        assert(psyqo::SIO0Bus::owned()&&pocket_sio_test::control==0);
        // Catch-up simulation ticks cannot shorten the physical recovery.
        assert(!request.poll(5000000)&&pocket_sio_test::writes==0);
        assert(!advance(request,269)&&pocket_sio_test::writes==0);
        assert(advance(request,1));
        assert(request.error()==PocketError::OK&&request.received()==5&&request.reply()[2]==2);
        assert(pocket_sio_test::writes==5&&!pocket_sio_test::pad_collision);
        assert(pocket_sio_test::other_irqs==0x49);
        assert(!psyqo::SIO0Bus::owned()&&pocket_sio_test::masked&&pocket_sio_test::control==0);
    }
    pocket_sio_test::reset(1);PocketRequest request;
    {
        psyqo::SIO0Bus::Lock filesystem;
        assert(request.begin(1,tx,sizeof(tx)));
        assert(request.error()==PocketError::Busy&&pocket_sio_test::writes==0);
        assert(psyqo::SIO0Bus::owned());
    }
    assert(!psyqo::SIO0Bus::owned());
    pocket_sio_test::reset(1);pocket_sio_test::ack=false;
    assert(request.begin(1,tx,sizeof(tx))&&!advance(request));
    assert(request.received()==1&&psyqo::SIO0Bus::owned()&&pocket_sio_test::writes==1);
    assert(!advance(request));
    pocket_sio_test::ack=true;
    assert(advance(request)&&request.error()==PocketError::OK&&pocket_sio_test::writes==7);
    assert(!psyqo::SIO0Bus::owned());
    // No card and no RX both terminate after three attempts with clean state.
    for(bool receive:{false,true}){
        pocket_sio_test::reset(1);pocket_sio_test::ack=false;pocket_sio_test::receive=receive;
        assert(request.begin(1,tx,sizeof(tx))&&!advance(request)&&!advance(request));
        assert(advance(request)&&request.error()==PocketError::NoCard);
        assert(request.received()==unsigned(receive)&&pocket_sio_test::writes==3);
        assert(!psyqo::SIO0Bus::owned()&&pocket_sio_test::masked&&pocket_sio_test::control==0);
    }
    // Once the command byte was sent, a failed reply must never replay it.
    pocket_sio_test::reset(1);pocket_sio_test::ack_bytes=1;
    const uint8_t launch[]={0x81,0x59,0,0,7,0,0,0,0};
    assert(request.begin(1,launch,sizeof(launch))&&advance(request));
    assert(request.received()==2&&pocket_sio_test::writes==2&&!psyqo::SIO0Bus::owned());
    // Preserve an initially masked-out controller IRQ and handle HSync wrap.
    pocket_sio_test::reset(0);pocket_sio_test::masked=false;pocket_sio_test::counters[1].value=65500;
    assert(request.begin(0,tx,sizeof(tx))&&advance(request)&&request.error()==PocketError::OK);
    assert(!pocket_sio_test::masked);
    // Ownership survives moves and is released by scene reset/destruction.
    pocket_sio_test::reset(1);
    {
        PocketRequest original;assert(original.begin(1,tx,sizeof(tx)));
        PocketRequest moved=static_cast<PocketRequest&&>(original);
        assert(!original.busy()&&psyqo::SIO0Bus::owned());
        request=static_cast<PocketRequest&&>(moved);
        assert(!moved.busy()&&psyqo::SIO0Bus::owned()&&advance(request));
        assert(original.begin(1,tx,sizeof(tx)));
        original=PocketRequest{};assert(!psyqo::SIO0Bus::owned());
        assert(original.begin(1,tx,sizeof(tx)));
    }
    assert(!psyqo::SIO0Bus::owned());
}
