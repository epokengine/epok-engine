#include "../../runtime/pocketstation_async.hpp"
#include <cassert>
int main(){
    auto& box=epok_pocket_mailbox;box.magic=epok::PocketMailbox::magic_value;
    epok::PocketRequest request;
    const uint8_t tx[]={0x81,0x58,0,0,0};
    assert(request.begin(1,tx,sizeof(tx)));assert(request.busy()&&!request.ready());
    assert(box.port==1&&box.frames==1&&box.lengths[0]==5&&box.send[1]==0x58);
    assert(!request.begin(0,tx,5));assert(!request.poll(16667));
    box.reply[0]=0xff;box.reply[2]=2;box.received[0]=5;box.answered=box.request;
    assert(request.poll(16667)&&request.ready()&&!request.busy());
    assert(request.received()==5&&request.reply()[2]==2);
    assert(request.begin(0,tx,5));
    assert(request.poll(5000000)&&request.error()==epok::PocketError::Timeout);
    // A timed-out bridge still owns the mailbox until its late answer arrives.
    assert(!request.begin(0,tx,5));box.answered=box.request;
    assert(request.begin(0,tx,5));box.received[0]=65535;box.answered=box.request;
    assert(request.poll(1)&&request.received()==5);
    assert(!request.begin(2,tx,5)&&request.error()==epok::PocketError::BadPort);
    assert(!request.begin(0,tx,141)&&request.error()==epok::PocketError::BadArgument);
    assert(request.begin(0,tx,5));box.received[0]=0;box.answered=box.request;
    assert(request.poll(1)&&request.error()==epok::PocketError::NoCard);
}
