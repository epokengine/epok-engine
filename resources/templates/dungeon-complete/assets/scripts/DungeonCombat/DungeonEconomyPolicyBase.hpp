#pragma once
#include "epok.hpp"
#include "device_io.hpp"
#include "pocketstation_apps.hpp"

#include "DungeonRulesBinding.hpp"
#include "DungeonInventory.hpp"

// Data and copy operations only; all policy decisions belong to the selected flavor.
struct EPOK_VALUE(Configuration,Id="b2e629fe-02a3-5beb-a202-cb256ab0bf02") DungeonSupplyReceipt {uint32_t serial=0;uint32_t total=0;};
struct EPOK_VALUE(Id="b1636db5-b1a4-573e-9893-9b37b8a8aacd") DungeonSupplyTransfer {int32_t result=0;uint32_t amount=0;};
struct EPOK_VALUE(Configuration,Id="695a170f-2eeb-5a19-9483-567ab2e1123d") DungeonEconomySave {
    uint32_t signature=1095980081;
    uint32_t version=1;
    uint32_t ore=0;
    uint32_t credits=0;
    uint32_t medkits=0;
    uint32_t tanks=0;
    uint32_t buster=0;
    DungeonSupplyReceipt r0{};
    DungeonSupplyReceipt r1{};
    DungeonSupplyReceipt r2{};
    DungeonSupplyReceipt r3{};
    DungeonSupplyReceipt r4{};
    DungeonSupplyReceipt r5{};
    DungeonSupplyReceipt r6{};
    DungeonSupplyReceipt r7{};
};
static_assert(sizeof(DungeonEconomySave)==92,"Existing seven words plus eight serial/total pairs");
struct EPOK_VALUE(Configuration,Id="cb9809ed-7ab9-5bd6-8dc0-c43f5b9f0575") DungeonSupplyTuning {
    uint32_t balance_limit=1000000;
    uint32_t medkit_cap=9;
    uint32_t tank_cap=9;
    uint32_t buster_cap=3;
    uint32_t sale_rate=10;
    uint32_t medkit_price=30;
    uint32_t tank_price=80;
    uint32_t buster_price=120;
    uint32_t medkit_heal=35;
    uint32_t tank_heal=100;
    uint32_t buster_damage=8;
};
struct EPOK_VALUE(Configuration,Id="54399f33-851d-5f59-9f74-30b5b31bad27") DungeonSupplyTransaction {
    uint32_t phase=0;uint32_t request=0;uint32_t serial=0;uint32_t total=0;uint32_t heal=0;
    bool ack_pending=false;epok::Fixed ack_time=0.0;
};
struct EPOK_VALUE(Id="95302fe0-cde6-58f6-a74f-665a83a0d73e") DungeonSupplyDecision {
    uint32_t action=0;uint32_t notice=0;int32_t result=0;uint32_t amount=0;uint32_t heal=0;
    uint32_t serial=0;uint32_t total=0;bool changed=false;
};
struct EPOK_VALUE(Configuration,Id="7f9bc0f6-1281-5016-a52c-e091b958cb54") DungeonPocketFlow {
    uint32_t operation=0;
    uint32_t port=0;
    uint32_t epoch=0;
    uint32_t installed=0;
    uint32_t error=0;
    uint32_t progress=101;
};
struct EPOK_VALUE(Id="e9bd9597-2d7d-53df-97a0-de677a728438") DungeonPocketObservation {
    bool ready=false;
    uint32_t epoch=0;
    uint32_t error=0;
    uint32_t running_block=0;
    uint32_t expected_block=0;
    uint32_t serial=0;
    uint32_t signature=0;
    uint32_t version=0;
    uint32_t total=0;
    uint32_t claimed=0;
    uint32_t percent=0;
};
class EPOK_CLASS(Abstract,Blueprintable,Owners=World3D,Id="be00ccfe-646f-57fd-b648-c3b69651b082") DungeonEconomyPolicyBase:public epok::ActorComponent {
public:
private:
    uint32_t wire_words_[32]{};
    uint8_t wire_bytes_[2664]{};
    epok::PocketTransferHandle wire_token_{};
    const epok::PocketApp* wire_image_=epok::find_pocket_app("BEEPOKPASTEROID");
public:
    void wire_assign_image(const epok::PocketApp& image){wire_image_=&image;}
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::UIActor",Id="e3dd4fa8-2584-5b64-b270-eee113cd88f0") epok::ObjectId hud_owner{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="ea7b21b3-9bf0-5ae2-a1f3-3804894bee24") epok::ObjectId pocket_station{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="ccec610a-2b7d-55a0-a2c9-16b63d2fa6ff") epok::ObjectId trading_station{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::PocketTransactionComponent",Id="880a9390-dd3b-5ec0-8230-ccf68909d3f8") epok::ObjectId pocket_channel{};
    epok::PocketTransactionComponent* wire_channel()const{return epok::active_object_registry?epok::active_object_registry->resolve<epok::PocketTransactionComponent>(pocket_channel):nullptr;}
    EPOK_FUNCTION(BlueprintPure,Id="af172e8c-e679-536c-a10f-6b8022fc5340") uint32_t wire_word(uint32_t index)const{return index<32?wire_words_[index]:0;}
    EPOK_FUNCTION(BlueprintCallable,Id="85cbfadd-dd47-5ae5-8cae-d1bb721d58ff") void wire_set_word(uint32_t index,uint32_t value){if(index<32)wire_words_[index]=value;}
    EPOK_FUNCTION(BlueprintPure,Id="019345f2-a92e-58f9-9d70-dc9be50a7772") uint32_t wire_byte(uint32_t index)const{return index<2664?wire_bytes_[index]:0;}
    EPOK_FUNCTION(BlueprintCallable,Id="7c79841c-6277-5053-b669-814c615f4a1d") void wire_set_byte(uint32_t index,uint32_t value){if(index<2664&&value<=255)wire_bytes_[index]=uint8_t(value);}
    EPOK_FUNCTION(BlueprintPure,Id="05c0fe60-05d5-5e3d-b3ba-637db049d681") uint32_t wire_u32(uint32_t index)const{if(index>2664-4)return 0;uint32_t word=0;for(uint32_t k=0;k<4;++k)word|=uint32_t(wire_bytes_[index+k])<<(8*k);return word;}
    EPOK_FUNCTION(BlueprintCallable,Id="847b8c9f-164c-5e7c-ac34-f6ec1d6e3718") void wire_set_u32(uint32_t index,uint32_t value){if(index>2664-4)return;for(uint32_t k=0;k<4;++k)wire_bytes_[index+k]=uint8_t(value>>(8*k));}
    EPOK_FUNCTION(BlueprintPure,Id="8c0075cc-e065-5c85-830b-bbc5c0ff5c35") bool wire_app_available()const{return wire_image_&&wire_image_->image&&wire_image_->filename;}
    EPOK_FUNCTION(BlueprintPure,Id="eac72795-876d-5d91-8312-b66b6ef2b125") uint32_t wire_app_size()const{return wire_image_?wire_image_->size:0;}
    EPOK_FUNCTION(BlueprintPure,Id="e6f3bb0c-38d6-5f89-b9aa-4dfa20b083a8") uint32_t wire_app_save_offset()const{return wire_image_?wire_image_->save_offset:0;}
    EPOK_FUNCTION(BlueprintPure,Id="656f1c9c-dae1-58da-8b92-d4b5605c699c") uint32_t wire_app_image_byte(uint32_t index)const{return wire_image_&&wire_image_->image&&index<wire_image_->size?wire_image_->image[index]:0;}
    EPOK_FUNCTION(BlueprintPure,Id="7766284e-fb2f-5daf-8bf1-52509353a5a7") uint32_t wire_app_name_byte(uint32_t index)const{if(!wire_image_||!wire_image_->filename||index>20)return 0;for(uint32_t k=0;k<index;++k)if(!wire_image_->filename[k])return 0;return uint8_t(wire_image_->filename[index]);}
    EPOK_FUNCTION(BlueprintPure,Id="ada042f5-44ca-5474-b255-a4851ce83b76") bool wire_channel_exists()const{return wire_channel()!=nullptr;}
    EPOK_FUNCTION(BlueprintCallable,Id="ada04db4-5e53-5729-8d68-74b1deff1214") void wire_configure_channel(uint32_t port,bool automatic_poll){if(auto* channel=wire_channel()){channel->port=port;channel->automatic_poll=automatic_poll;}}
    EPOK_FUNCTION(BlueprintPure,Id="7ff7839f-d0f8-57f2-bf7c-11ff27a16b94") epok::PocketTransferHandle wire_token()const{return wire_token_;}
    EPOK_FUNCTION(BlueprintCallable,Id="af1dbd62-af55-5595-9c03-686d7d3177f7") void wire_assign_token(epok::PocketTransferHandle value){wire_token_=value;}
    EPOK_FUNCTION(BlueprintPure,Id="9102cf11-a0ea-5511-b86d-dc139a91378e") uint32_t wire_error_code(epok::PocketError value)const{return uint32_t(value);}
    EPOK_FUNCTION(BlueprintPure,Id="86a806b5-1fd0-5ab0-8796-497b69bd2e75") uint32_t wire_epoch()const{return pocket.epoch;}
    EPOK_FUNCTION(BlueprintPure,Id="8921a6d4-5711-5b73-abc5-b34b53e6dafb") DungeonPocketObservation wire_make_observation(bool ready,uint32_t epoch,uint32_t error,uint32_t running_block,uint32_t expected_block,uint32_t serial,uint32_t signature,uint32_t version,uint32_t total,uint32_t claimed,uint32_t percent)const{return {ready,epoch,error,running_block,expected_block,serial,signature,version,total,claimed,percent};}
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="2450ac09-1bc0-5adf-b6f2-ac22e4b165c7") virtual DungeonSupplyDecision pocket_execute(uint32_t action,uint32_t port)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="9017cece-2715-5921-8405-58e633e4ff8d") virtual DungeonSupplyDecision pocket_update(epok::Fixed dt,uint32_t microseconds)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="4b32094f-11c8-508f-be3f-648b81274545") virtual void protocol_cancel()=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="2afef0cf-07af-5269-afa1-a2d8125689a5") virtual DungeonPocketObservation protocol_observation()=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="13fe9495-8d12-5b17-885a-e66ba20e1e6e") virtual uint32_t protocol_percent()=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="5295716f-d358-5ff0-a638-215166769d5d") virtual bool protocol_start(uint32_t mode,uint32_t amount,uint32_t expected_serial)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="c37da1d4-ed29-5039-bb82-dc2ae67d70a6") virtual bool protocol_tick(uint32_t microseconds)=0;
    EPOK_PROPERTY(Id="0eeb2e9a-b9c4-5135-8a69-8cb08d0ec283") DungeonPocketFlow pocket{0,0,0,0,0,101};
    EPOK_FUNCTION(BlueprintCallable,Id="cbb7cc0e-6c5b-52f8-ba4d-37143a167de9") void assign_pocket(DungeonPocketFlow value){pocket=value;}
    EPOK_FUNCTION(BlueprintPure,Id="91b52a5b-bddd-5828-9229-88e941e8ec82") DungeonPocketFlow make_pocket(uint32_t operation,uint32_t port,uint32_t epoch,uint32_t installed,uint32_t error,uint32_t progress)const{return {operation,port,epoch,installed,error,progress};}
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="8f768c38-06c9-5ad2-99f2-ce378bf8b5da") virtual DungeonSupplyDecision pocket_begin(uint32_t action,bool app_available,uint32_t port)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="c6b937fb-5078-598e-8892-9c2354d193fc") virtual DungeonSupplyDecision pocket_progress(uint32_t epoch,uint32_t percent)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="521349e2-2f33-5381-b629-40c5ae50b42a") virtual DungeonSupplyDecision pocket_request_failed(uint32_t epoch,uint32_t error)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="ceff9044-4613-5a1e-aaa4-29ab1e3aa49a") virtual DungeonSupplyDecision pocket_observe(DungeonPocketObservation sample)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="723e7e03-1ebd-5672-affb-be91aaa318d1") virtual DungeonSupplyDecision pocket_advance(epok::Fixed dt)=0;
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("be00ccfe-646f-57fd-b648-c3b69651b082");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    // Common project schema/metadata copies. Chosen-language rules decide when
    // to stage, queue, accept and acknowledge a transaction.
    EPOK_FUNCTION(BlueprintCallable,Id="ff1d571e-bd30-5c3e-980c-9e5ab034955c") void stage_card_metadata(){
        epok::MemoryCardLibrary::clear_staged_metadata();
        for(uint32_t n=0;n<sizeof(dungeon_inventory::file_name);++n)epok::MemoryCardLibrary::set_staged_name_byte(n,uint8_t(dungeon_inventory::file_name[n]));
        for(uint32_t n=0;n<sizeof(dungeon_inventory::file_title);++n)epok::MemoryCardLibrary::set_staged_title_byte(n,uint8_t(dungeon_inventory::file_title[n]));
    }
    EPOK_FUNCTION(BlueprintCallable,Id="d78d5492-6f61-5dad-b374-322788f78643") void stage_pending_payload(){
        const auto* bytes=reinterpret_cast<const uint8_t*>(&pending);
        for(uint32_t n=0;n<sizeof(pending)/4;++n){uint32_t word=0;for(uint32_t k=0;k<4;++k)word|=uint32_t(bytes[n*4+k])<<(k*8);epok::MemoryCardLibrary::set_staged_word(n,word);}
    }
    EPOK_FUNCTION(BlueprintPure,Id="39777e94-ada7-5d21-be70-f7be1d2344eb") DungeonEconomySave loaded_payload()const{
        DungeonEconomySave value{};auto* bytes=reinterpret_cast<uint8_t*>(&value);
        for(uint32_t n=0;n<sizeof(value)/4;++n){const auto word=epok::MemoryCardLibrary::loaded_word(n);for(uint32_t k=0;k<4;++k)bytes[n*4+k]=uint8_t(word>>(k*8));}
        return value;
    }
    EPOK_PROPERTY(EditAnywhere,Id="a407b470-d874-54ed-93c1-43b39634df89") DungeonSupplyTuning tuning{1000000,9,9,3,10,30,80,120,35,100,8};
    EPOK_PROPERTY(Id="bd9711fd-2837-5c7d-bf49-d1ef30271b8a") DungeonEconomySave saved{1095980081,1,0,0,0,0,0,{0,0},{0,0},{0,0},{0,0},{0,0},{0,0},{0,0},{0,0}};
    EPOK_PROPERTY(Id="84718536-1b7a-544d-9eb2-be963657d8d1") DungeonEconomySave pending{1095980081,1,0,0,0,0,0,{0,0},{0,0},{0,0},{0,0},{0,0},{0,0},{0,0},{0,0}};
    EPOK_PROPERTY(Id="565db572-5359-5812-87ae-a74cd58860ea") DungeonSupplyTransaction tx{0,0,0,0,0,false,0.0};
    EPOK_PROPERTY(Id="7acfa417-162b-5f19-9a3e-08ec2d9ff178") bool loaded=false;
    EPOK_FUNCTION(BlueprintCallable,Id="e75b4099-01d2-51e9-822b-7e57730dcd89") void assign_saved(DungeonEconomySave value){saved=value;}
    EPOK_FUNCTION(BlueprintCallable,Id="940c3aa6-02d0-5744-8c15-c80cb998856b") void assign_pending(DungeonEconomySave value){pending=value;}
    EPOK_FUNCTION(BlueprintCallable,Id="49aea1f4-34ef-5a5f-b998-02121281a0a9") void assign_tx(DungeonSupplyTransaction value){tx=value;}
    EPOK_FUNCTION(BlueprintCallable,Id="1a9c42d4-2356-541b-b13b-172a7a678f02") void assign_loaded(bool value){loaded=value;}
    EPOK_FUNCTION(BlueprintPure,Id="2dc9e3ae-42d6-55d6-8eb4-7177c877d4be") DungeonEconomySave empty_save()const{return {};}
    EPOK_FUNCTION(BlueprintCallable,Id="49453e81-f3cf-5886-8c8f-c1ae5e1bd6c6") void wallet(uint32_t ore,uint32_t credits,uint32_t medkits,uint32_t tanks,uint32_t buster){pending.ore=ore;pending.credits=credits;pending.medkits=medkits;pending.tanks=tanks;pending.buster=buster;}
    EPOK_FUNCTION(BlueprintPure,Id="da90e4a0-ec95-5c21-a179-a90ca44ceca2") DungeonSupplyTransaction make_tx(uint32_t phase,uint32_t request,uint32_t serial,uint32_t total,uint32_t heal,bool ack_pending,epok::Fixed ack_time)const{return {phase,request,serial,total,heal,ack_pending,ack_time};}
    EPOK_FUNCTION(BlueprintPure,Id="7244ce49-9ab7-58e5-befc-79a8a1628c93") DungeonSupplyTransfer make_transfer(int32_t result,uint32_t amount)const{return {result,amount};}
    EPOK_FUNCTION(BlueprintPure,Id="d4b39447-56d7-5abe-ac0b-e2f40b6c3aad") DungeonSupplyDecision decision(uint32_t action,uint32_t notice,int32_t result,uint32_t amount,uint32_t heal,uint32_t serial,uint32_t total,bool changed)const{return {action,notice,result,amount,heal,serial,total,changed};}
    EPOK_FUNCTION(BlueprintPure,Id="7c48a73c-51ab-5a6f-939f-f7f9fb4d3f22") DungeonSupplyReceipt pending_receipt(uint32_t index)const{switch(index){case 0:return pending.r0;case 1:return pending.r1;case 2:return pending.r2;case 3:return pending.r3;case 4:return pending.r4;case 5:return pending.r5;case 6:return pending.r6;case 7:return pending.r7;default:return {};}}
    EPOK_FUNCTION(BlueprintPure,Id="349083ec-b127-583f-b68d-6e0a13b90a10") DungeonSupplyReceipt saved_receipt(uint32_t index)const{switch(index){case 0:return saved.r0;case 1:return saved.r1;case 2:return saved.r2;case 3:return saved.r3;case 4:return saved.r4;case 5:return saved.r5;case 6:return saved.r6;case 7:return saved.r7;default:return {};}}
    EPOK_FUNCTION(BlueprintCallable,Id="1f8ad779-cab4-5497-98ca-5b9d38c871b9") void write_receipt(uint32_t index,uint32_t serial,uint32_t total){switch(index){case 0:pending.r0={serial,total};break;case 1:pending.r1={serial,total};break;case 2:pending.r2={serial,total};break;case 3:pending.r3={serial,total};break;case 4:pending.r4={serial,total};break;case 5:pending.r5={serial,total};break;case 6:pending.r6={serial,total};break;case 7:pending.r7={serial,total};break;default:break;}}
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="4d8939bf-90e0-55c0-b07b-f37a4c2123c4") virtual bool valid(DungeonEconomySave value)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="c71c5465-1aff-579e-9c2a-1680fc6f11a1") virtual uint32_t price(DungeonEconomySave value,uint32_t item)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="4c436057-136a-58a4-8703-7fff62fb3536") virtual uint32_t damage_bonus()const=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="ee97a117-34fd-531f-8624-47064c416439") virtual uint32_t find_receipt(uint32_t serial)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="931be0fa-b914-58bf-8a09-771d7d803c8c") virtual DungeonSupplyTransfer transfer(uint32_t serial,uint32_t total,uint32_t claimed)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="eafad74c-517a-52e2-a707-5c74d142b274") virtual bool sell()=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="fbe484c6-01b9-524f-b7db-4ccee25728f4") virtual bool buy(uint32_t item)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="fb33024d-9136-536b-8498-45fa48a203df") virtual DungeonSupplyDecision activate(uint32_t menu,uint32_t selection,bool can_heal)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="e79d40ad-e0ba-5bde-a479-8b9487c46823") virtual DungeonSupplyDecision receive(bool running,uint32_t signature,uint32_t version,uint32_t serial,uint32_t total,uint32_t claimed)=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="bbf0b5f6-c344-5c01-b788-7685c1d74d02") virtual bool can_save()const=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="954d1f57-e603-5b8c-aaaf-9029608814c7") virtual DungeonSupplyDecision save_started(uint32_t request,bool accepted)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="23fea38d-cc5d-538a-abb5-ba8af4361966") virtual DungeonSupplyDecision save_completed(uint32_t request,bool succeeded)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="cdca925a-b2b3-5e54-af4b-b9776221dbc4") virtual DungeonSupplyDecision ack_started(bool accepted)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="1f7265cf-a3cc-5f0e-a0cd-037a449addb6") virtual DungeonSupplyDecision ack_sent(bool succeeded)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="35a44dd2-0b20-5234-ad83-692435712767") virtual DungeonSupplyDecision advance_ack(epok::Fixed dt)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="c1b956d2-91b8-5ced-9e71-d6f6a7a5ce60") virtual DungeonSupplyDecision receipt_checked(bool succeeded,uint32_t serial,uint32_t signature,uint32_t claimed)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="d55ed8fe-7945-5552-becf-830060a57980") virtual DungeonSupplyDecision load_result(DungeonEconomySave candidate,bool succeeded,bool missing)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="54947612-88c2-5448-83bc-876a0d3c9df4") virtual DungeonSupplyDecision leave()=0;
};
using DungeonEconomyPolicyBinding=DungeonRulesBinding<DungeonEconomyPolicyBase>;
