#pragma once
#include "pocketstation_async.hpp"

// One resident transaction state machine; no actors, allocation or spin waits.
class DungeonPocketLink {
public:
    enum Mode:uint8_t {Install,Inspect,Receipt,ReceiptStatus};
    epok::PocketStatus status{};
    int32_t values[16]{};
    uint8_t block=0;
    epok::PocketError error=epok::PocketError::OK;
    const char* detail=nullptr;
private:
    enum Phase:uint8_t {None,Probe,Directory,Status,Shared,Confirm,Data,Commit,Launch,Send,
        UpgradeStatus,Stop,Stopped,Existing,Identity,Verify,Retire,StartForTransfer,AwaitStart};
    epok::PocketRequest request;
    const epok::PocketApp* app=nullptr;
    uint8_t directory[16][128]{},blocks[15]{};
    unsigned port=0,index=0,needed=0,sector=0,completed_sectors=0;
    uint32_t expected_serial=0,total=0;
    Mode mode=Install;Phase phase=None;bool complete=false;
    uint8_t old_block=0,save_data[128]{};
    bool upgrading=false,starting_transfer=false;
    unsigned stop_polls=0,start_polls=0,read_address=0;
    uint32_t legacy_hash=0,current_hash=0;
    // Exact shipped v1 program, excluding its mutable Pocket Lua save sector.
    static constexpr unsigned legacy_save_sector=42;
    static constexpr uint32_t legacy_fingerprint=0x99022a33u;
    static uint32_t get32(const uint8_t* p){return uint32_t(p[0])|uint32_t(p[1])<<8|uint32_t(p[2])<<16|uint32_t(p[3])<<24;}
    static void put32(uint8_t* p,uint32_t v){for(unsigned i=0;i<4;++i)p[i]=uint8_t(v>>(8*i));}
    void fail(epok::PocketError e){error=e;phase=None;complete=true;}
    void finish(){fail(epok::PocketError::OK);}
    void reject(const char* message){detail=message;fail(epok::PocketError::BadReply);}
    void begin(Phase next,const uint8_t* bytes,unsigned n,int at=-1){phase=next;request.begin(port,bytes,uint16_t(n),int16_t(at));}
    void read_directory(){read_sector(Directory,index);}
    void read_sector(Phase next,unsigned address){
        read_address=address;const uint8_t bytes[140]={0x81,0x52,0,0,uint8_t(address>>8),uint8_t(address)};
        begin(next,bytes,140);
    }
    bool valid_sector(){
        const auto* r=request.reply();uint8_t checksum=uint8_t(read_address>>8)^uint8_t(read_address);
        if(request.received()!=140||r[139]!=0x47||r[8]!=uint8_t(read_address>>8)||r[9]!=uint8_t(read_address)){
            fail(epok::PocketError::BadReply);return false;
        }
        for(unsigned i=0;i<128;++i)checksum^=r[10+i];
        if(checksum!=r[138]){fail(epok::PocketError::BadReply);return false;}return true;
    }
    void read_status(Phase next=Status){const uint8_t bytes[21]={0x81,0x5a};begin(next,bytes,21);}
    void read_values(){const uint8_t bytes[70]={0x81,0x5b,0x82};begin(Shared,bytes,70,4);}
    void launch(){const uint8_t bytes[9]={0x81,0x59,0,0,block};begin(Launch,bytes,9);}
    void wait_for_start(){
        if(++start_polls>120){reject("Asteroid Catcher did not become ready. Try again.");return;}
        read_status(AwaitStart);
    }
    void write_sector(Phase next,unsigned address,const uint8_t* data){
        uint8_t bytes[138]={0x81,0x57,0,0,uint8_t(address>>8),uint8_t(address)};
        uint8_t checksum=bytes[4]^bytes[5];
        for(unsigned i=0;i<128;++i){bytes[6+i]=data[i];checksum^=data[i];}
        bytes[134]=checksum;begin(next,bytes,138);
    }
    const uint8_t* data_source()const{
        return upgrading&&!index&&sector==app->save_offset/128?save_data:app->image+index*8192+sector*128;
    }
    void write_data(){write_sector(Data,unsigned(blocks[index])*64+sector,data_source());}
    bool named(const uint8_t* stored)const{
        for(unsigned i=0;i<20;++i){if(stored[i]!=uint8_t(app->filename[i]))return false;if(!stored[i])return true;}
        return !app->filename[20];
    }
    void allocate(){
        needed=app->size/8192;unsigned found=0;
        for(uint8_t b=1;b<16&&found<needed;++b)if((directory[b][0]&0xf0)==0xa0)blocks[found++]=b;
        if(found!=needed){detail=old_block?"Update needs one free block. Existing ore kept.":nullptr;fail(epok::PocketError::CardFull);return;}
        block=blocks[0];index=sector=0;write_data();
    }
    void inspect_existing(){
        sector=0;legacy_hash=current_hash=2166136261u;read_sector(Existing,unsigned(old_block)*64);
    }
    static uint32_t fingerprint(const epok::PocketApp& image){
        uint32_t hash=2166136261u;
        for(unsigned i=0;i<image.size;++i)if(i/128!=image.save_offset/128)hash=(hash^image.image[i])*16777619u;
        return hash;
    }
    void directory_ready(){
        unsigned matches=0;for(uint8_t b=1;b<16;++b)if(directory[b][0]==0x51&&named(directory[b]+10))++matches;
        if(matches>1){reject("Multiple minigame copies. Data kept; update stopped.");return;}
        for(uint8_t b=1;b<16;++b)if(directory[b][0]==0x51&&named(directory[b]+10)){
            block=b;
            if(mode!=Install)read_status();
            else if(!app->save_offset)launch();
            else if(app->size!=8192||get32(directory[b]+4)!=8192||directory[b][8]!=0xff||directory[b][9]!=0xff)
                reject("Unsupported minigame layout. Existing data kept.");
            else{old_block=b;read_status(UpgradeStatus);}
            return;
        }
        if(mode!=Install){fail(epok::PocketError::NotInstalled);return;}
        allocate();
    }
    void prepare_commit(){
        for(unsigned i=0;i<needed;++i){
            auto* frame=directory[blocks[i]];for(unsigned c=0;c<128;++c)frame[c]=0;
            frame[0]=i==0?0x51:(i+1==needed?0x53:0x52);
            if(!i){put32(frame+4,app->size);for(unsigned c=0;c<20&&app->filename[c];++c)frame[10+c]=uint8_t(app->filename[c]);}
            const uint16_t next=i+1<needed?uint16_t(blocks[i+1]-1):0xffff;
            frame[8]=uint8_t(next);frame[9]=uint8_t(next>>8);
            for(unsigned c=0;c<127;++c)frame[127]^=frame[c];
        }
        // Publish the head last: an interrupted fresh install has no head
        // pointing at incomplete program data. An upgrade retains the old app
        // until the replacement has been read back and its head committed.
        index=needed-1;write_sector(Commit,blocks[index],directory[blocks[index]]);
    }
    bool parse_status(epok::PocketStatus& out){
        const auto* r=request.reply();
        if(request.received()!=21||r[2]!=0x12){fail(epok::PocketError::NotPocketStation);return false;}
        out.running_block=uint16_t(uint16_t(r[3])<<8|r[4]);out.serial=get32(r+9);return true;
    }
public:
    bool done()const{return complete;}
    unsigned percent()const{
        if(complete&&error==epok::PocketError::OK)return 100;
        return needed?completed_sectors*99/(needed*64):0;
    }
    void start(Mode operation,const epok::PocketApp& image,unsigned slot,uint32_t amount=0,uint32_t serial=0){
        if(request.busy())return;
        mode=operation;app=&image;port=slot;total=amount;expected_serial=serial;
        error=epok::PocketError::OK;detail=nullptr;complete=false;block=old_block=0;upgrading=starting_transfer=false;
        index=sector=needed=completed_sectors=stop_polls=start_polls=0;status={};
        if(!app->image||!app->filename||!app->size||app->size%8192||app->size>15*8192){fail(epok::PocketError::BadArgument);return;}
        if(app->save_offset&&(app->save_offset%128||app->save_offset+128>app->size)){fail(epok::PocketError::BadArgument);return;}
        if(mode==Receipt||mode==ReceiptStatus)read_status();
        else{const uint8_t bytes[5]={0x81,0x58};begin(Probe,bytes,5);}
    }
    void tick(uint32_t microseconds){
        if(complete||phase==None||!request.poll(microseconds))return;
        if(request.error()!=epok::PocketError::OK){fail(request.error());return;}
        const auto* r=request.reply();
        switch(phase){
        case UpgradeStatus:
            if(!parse_status(status))break;
            if(status.running_block==old_block){const uint8_t bytes[9]={0x81,0x59,0,0,0,uint8_t(0x30+old_block)};begin(Stop,bytes,9);}
            else inspect_existing();
            break;
        case Stop:
            if(request.received()!=9||r[2]!=6){fail(epok::PocketError::BadReply);break;}
            read_status(Stopped);break;
        case Stopped:{
            epok::PocketStatus current{};if(!parse_status(current))break;
            if(current.serial!=status.serial){reject("Card changed. Existing data kept.");break;}
            if(current.running_block==old_block){
                if(++stop_polls>=120){fail(epok::PocketError::Running);break;}
                read_status(Stopped);
            }else inspect_existing();
            break;
        }
        case Existing:
            if(!valid_sector())break;
            for(unsigned i=0;i<128;++i){
                if(sector!=legacy_save_sector)legacy_hash=(legacy_hash^r[10+i])*16777619u;
                else save_data[i]=r[10+i];
                if(sector!=app->save_offset/128)current_hash=(current_hash^r[10+i])*16777619u;
            }
            if(++sector<64){read_sector(Existing,unsigned(old_block)*64+sector);break;}
            if(current_hash==fingerprint(*app)){launch();break;}
            if(legacy_hash!=legacy_fingerprint){reject("Unknown minigame version. Existing data kept.");break;}
            if(get32(save_data)>1000000||get32(save_data+4)>1000000||get32(save_data+8)>get32(save_data+4)){
                reject("Invalid minigame save. Existing data kept.");break;
            }
            upgrading=true;read_status(Identity);break;
        case Identity:{
            epok::PocketStatus current{};if(!parse_status(current))break;
            if(current.serial!=status.serial){reject("Card changed. Existing data kept.");break;}
            if(current.running_block==old_block){fail(epok::PocketError::Running);break;}
            allocate();break;
        }
        case Verify:
            if(!valid_sector())break;
            for(unsigned i=0;i<128;++i)if(r[10+i]!=data_source()[i]){reject("Verification failed. Existing data kept.");return;}
            if(++sector==64){sector=0;++index;}
            if(index<needed)read_sector(Verify,unsigned(blocks[index])*64+sector);else prepare_commit();
            break;
        case Retire:
            if(request.received()!=138||r[137]!=0x47){fail(epok::PocketError::Refused);break;}
            launch();break;
        case Probe:
            if(request.received()!=5||r[2]!=2){fail(epok::PocketError::NotPocketStation);break;}
            index=0;read_directory();break;
        case Directory:{
            if(!valid_sector())break;
            uint8_t checksum=0;for(unsigned i=0;i<128;++i){directory[index][i]=r[10+i];if(i<127)checksum^=r[10+i];}
            if(checksum!=directory[index][127]){fail(epok::PocketError::BadReply);break;}
            if(!index&&(directory[0][0]!='M'||directory[0][1]!='C')){fail(epok::PocketError::NotFormatted);break;}
            if(++index<16)read_directory();else directory_ready();break;
        }
        case Status:
            if(!parse_status(status))break;
            if(expected_serial&&status.serial!=expected_serial){fail(epok::PocketError::BadReply);break;}
            if(mode==Inspect&&status.running_block!=block){
                if(status.running_block){reject("Open Asteroid Catcher to transfer. Another app is running.");break;}
                // The native menu has no app link functions. Start the installed
                // copy without rewriting flash, then wait across frames for init.
                starting_transfer=true;const uint8_t bytes[9]={0x81,0x59,0,0,block};begin(StartForTransfer,bytes,9);
            }
            else if(mode==Receipt){uint8_t bytes[12]={0x81,0x5c,0x80,0,5,0,0xd1};put32(bytes+7,total);begin(Send,bytes,12);}
            else read_values();break;
        case StartForTransfer:
            if(request.received()!=9||r[2]!=6){fail(epok::PocketError::BadReply);break;}
            wait_for_start();break;
        case AwaitStart:{
            epok::PocketStatus current{};if(!parse_status(current))break;
            if(current.serial!=status.serial){reject("PocketStation changed. Nothing transferred.");break;}
            if(current.running_block==block){status=current;read_values();}
            else if(!current.running_block)wait_for_start();
            else reject("Another app started. Nothing transferred.");
            break;
        }
        case Shared:
            if(request.received()!=70||r[2]!=0xff||r[3]!=0||r[4]!=64){
                if(starting_transfer)wait_for_start();else fail(epok::PocketError::BadReply);break;
            }
            for(unsigned i=0;i<16;++i)values[i]=int32_t(get32(r+5+i*4));
            if(starting_transfer&&(uint32_t(values[2])!=0x41535431u||values[4]!=1))wait_for_start();
            else read_status(Confirm);break;
        case Confirm:{
            epok::PocketStatus current{};if(!parse_status(current))break;
            if(current.serial!=status.serial||current.running_block!=status.running_block){fail(epok::PocketError::BadReply);break;}
            finish();break;
        }
        case Data:
        case Commit:
            if(request.received()!=138||r[137]!=0x47){fail(epok::PocketError::Refused);break;}
            if(phase==Data){
                ++completed_sectors;if(++sector==64){sector=0;++index;}
                if(index<needed)write_data();else{index=sector=0;read_sector(Verify,unsigned(blocks[0])*64);}
            }else if(index){--index;write_sector(Commit,blocks[index],directory[blocks[index]]);}
            else if(upgrading){
                auto* frame=directory[old_block];frame[0]=0xa1;frame[127]=0;
                for(unsigned i=0;i<127;++i)frame[127]^=frame[i];write_sector(Retire,old_block,frame);
            }else launch();
            break;
        case Launch:
            if(request.received()!=9||r[2]!=6)fail(epok::PocketError::BadReply);else finish();break;
        case Send:
            if(request.received()!=12||r[2]!=0xff||r[3]!=1||r[5]!=5)fail(epok::PocketError::BadReply);else finish();break;
        default:break;
        }
    }
};
