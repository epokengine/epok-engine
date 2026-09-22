#pragma once
// Engine-owned, allocation-free gameplay function libraries and plain values.
// Include epok.hpp when this header is used directly; when reached from the
// umbrella, #pragma once makes the cycle a no-op after the core declarations.
#define EPOK_INCLUDE_FROM_GAMEPLAY_API 1
#include "epok.hpp"
#undef EPOK_INCLUDE_FROM_GAMEPLAY_API
#include "blueprint_api.hpp"
#include "skeletal.hpp"
#include "world2d.hpp"
#include "utility.hpp"

namespace epok {

struct EPOK_VALUE(Id="b1f11269-310f-4e57-a1c2-ae8133d17a16") InputAxisSample {
    bool connected=false;
    bool analog=false;
    Fixed value=Fixed(0.0);
};

struct EPOK_VALUE(Id="705a2b8f-7aa1-455d-830c-1af43e2a4a52") TimeSnapshot {
    bool paused=false;
    uint32_t simulation_ticks=0;
    uint32_t dropped_steps=0;
    uint32_t frame_microseconds=0;
    Fixed fixed_delta=Fixed(0.0);
};

struct EPOK_VALUE(Id="1a95f3e5-4d5b-4bcb-90ac-34d7fb61bc77") GameplayVector3 {
    Fixed x=0.0,y=0.0,z=0.0;
};
struct EPOK_VALUE(Id="b8f68929-ed24-4c76-8fdd-52b785024d74") GameplayVector2 {
    Fixed x=0.0,y=0.0;
};
// A dead-zoned analog stick reduced to a planar direction and a strength.
// `x` and `y` are the stick vector rescaled so the dead zone maps to zero and a
// fully deflected stick maps to one; `strength` is that rescaled magnitude.
struct EPOK_VALUE(Id="0b90f02c-2962-475d-bc38-9d10ed8272c1") MovementIntent {
    Fixed x=0.0,y=0.0,strength=0.0;
};
struct EPOK_VALUE(Id="aeb84ccb-e308-456a-a59f-31bed0cb08a4") GameplayCamera2D {
    GameplayVector2 position{};
    Fixed zoom=1.0,rotation=0.0;
    int32_t viewport_x=0,viewport_y=0,viewport_width=320,viewport_height=240;
};
struct EPOK_VALUE(Id="cb5a92d0-066e-48dc-a3b4-b96cce6e019b") GameplayAabb {
    GameplayVector3 minimum{},maximum{};
};
struct EPOK_VALUE(Id="35dd8902-b9d6-43de-a14e-b536e009cdca") CollisionHitSample {
    bool hit=false,started_inside=false;
    ObjectId actor{};
    Fixed fraction=1.0;
    GameplayVector3 point{},normal{};
};
struct EPOK_VALUE(Id="bd820338-f2a1-43c0-80e0-aefbc9cc335b") MoveSample {
    bool valid=false,grounded=false,blocked=false,unresolved_overlap=false;
    ObjectId actor{};
    GameplayVector3 displacement{},normal{};
};
struct EPOK_VALUE(Id="46e7177d-8b8a-43a0-b0ef-dfb9de55cab8") SceneSnapshot {
    uint32_t active=0,transitions=0,rejected=0;
    bool loading=false,waiting=false;
};
struct EPOK_VALUE(Id="766c88ea-1017-4cd1-a26d-899541647078") GameplayTransitionOptions {
    uint32_t fade_out_ms=EPOK_FADE_OUT_MS,fade_in_ms=EPOK_FADE_IN_MS;
    uint32_t red=255,green=255,blue=255;
};
struct EPOK_VALUE(Id="dd229750-b0fe-43e5-a20f-83c53f30130a") GameplayTransitionSnapshot {
    TransitionPhase phase=TransitionPhase::Idle;
    uint32_t audio_gain=4096,opacity=0;
    bool busy=false,presented=false;
};
// Per-scene distance fog. Distances are named in full because `end` is a Lua
// keyword and a record member has to stay readable in every execution mode.
struct EPOK_VALUE(Id="8b4375d1-f9cf-431c-9194-b16612bcf12d") FogSettings {
    bool enabled=false;
    Fixed start_distance=12.0,end_distance=40.0;
    uint32_t red=64,green=77,blue=102;
};
struct EPOK_VALUE(Id="0748249f-e8e3-4f29-9670-4ecdb4704448") ResourceSnapshot {
    uint32_t alive_slots=0,active_slots=0,slot_capacity=0,scene_banks=0;
    uint32_t textures=0,resident_texture_bytes=0,active_texture_bytes=0;
    uint32_t particles=0,particle_peak=0,particle_dropped=0;
    uint32_t mesh_triangles=0,sprite_triangles=0,dropped_primitives=0,frame_scanlines=0;
};
struct EPOK_VALUE(Id="944596e6-f69a-4f33-889f-c9c7c15d3673") MemoryCardSnapshot {
    CardState state=CardState::Idle;
    CardOperation operation=CardOperation::None;
    CardError error=CardError::OK,last_rejection=CardError::OK;
    uint32_t request=0,completed=0,payload_bytes=0,file_count=0;
};
struct EPOK_VALUE(Id="f16d4050-4722-4377-99de-e0bd5d699407") SavePayload8 {
    uint32_t word0=0,word1=0,word2=0,word3=0,word4=0,word5=0,word6=0,word7=0;
};
struct EPOK_VALUE(Id="e05ac2d9-e97c-4e65-a9b9-b9110ac5e32f") CardFileSample {
    bool valid=false;
    uint32_t name_hash=0,blocks=0;
};
struct EPOK_VALUE(Id="462f5091-d1c4-4a92-9958-c65947469b59") ProjectedPoint {
    bool success=false;
    Fixed x=0.0,y=0.0;
};
struct EPOK_VALUE(Id="f24065e1-e5b0-4608-8863-a20e708fd4df") GameplayPlaybackSnapshot {
    bp::PlaybackResult result=bp::PlaybackResult::Cancelled;
    uint32_t revision=0;
};
struct EPOK_VALUE(Id="93318a22-af5b-49d8-a114-aaf101faab5c") SkeletalQuerySnapshot {
    uint32_t calls=0,vertices=0,bones=0,decoded_bytes=0,failures=0;
};
// A Vector3 tween is one clock plus two endpoints. `timing` is an ordinary scalar
// tween over 0..1, so `utilities.tween_value(timing)` is the interpolation
// parameter the three components share and each component is exactly what a
// scalar tween over that component would report. Nothing here duplicates the
// clock, the easing catalogue, the delay or the loop plan.
struct EPOK_VALUE(Id="c4306dbe-18ba-4f38-98bc-bdb558ed3667") GameplayVector3TweenState {GameplayVector3 from{},to{};GameplayTweenState timing{};};
struct EPOK_VALUE(Id="872b1e73-f320-4bf9-a6c3-e77ea6e92371") Vector3TweenAdvanceSample {GameplayVector3TweenState state{};GameplayVector3 value{};bool completed=false;};

inline GameplayVector3 gameplay_vector(const Fixed* value){return {value[0],value[1],value[2]};}
inline void gameplay_vector(GameplayVector3 value,Fixed* output){output[0]=value.x;output[1]=value.y;output[2]=value.z;}
inline ActorData* gameplay_actor_data(ObjectId id){auto* actor=active_object_registry?active_object_registry->resolve<Actor>(id):nullptr;return actor?actor->data():nullptr;}
inline ObjectId gameplay_actor_id(DataHandle handle){auto* data=handle.get();return data&&data->owner?data->owner->id():ObjectId{};}
inline const char* gameplay_card_slot_name(uint32_t slot){static const char* names[8]={"SAVE0","SAVE1","SAVE2","SAVE3","SAVE4","SAVE5","SAVE6","SAVE7"};return slot<8?names[slot]:nullptr;}
inline int16_t gameplay_i16(int32_t value){return int16_t(value<-32768?-32768:value>32767?32767:value);}
inline uint32_t* gameplay_save_words(){alignas(4) static uint32_t words[MemoryCardService::max_payload/4]{};return words;}
inline uint16_t gameplay_u16(uint32_t value){return uint16_t(value>65535?65535:value);}
inline uint8_t gameplay_u8(uint32_t value){return uint8_t(value>255?255:value);}
inline GameplayVector3TweenState gameplay_vector_tween(GameplayVector3 from,GameplayVector3 to,Tween timing){return {from,to,gameplay_tween_state(timing)};}
inline GameplayVector3 gameplay_vector_tween_value(GameplayVector3TweenState state){const auto alpha=gameplay_tween(state.timing).alpha();return {lerp(state.from.x,state.to.x,alpha),lerp(state.from.y,state.to.y,alpha),lerp(state.from.z,state.to.z,alpha)};}

struct EPOK_FUNCTION_LIBRARY(Category="Input", Id="d6963400-e105-4af5-826c-e4b2edfc09df") InputLibrary {
    EPOK_FUNCTION(BlueprintPure, Id="8c11a90e-5791-40f8-89d7-1092c6376b48")
    static bool connected(uint32_t port) { return input.connected(port); }
    EPOK_FUNCTION(BlueprintPure, Id="d46fcfd7-f356-4f90-a2cc-271cff74391e")
    static bool analog(uint32_t port) { return input.analog(port); }
    EPOK_FUNCTION(BlueprintPure, Id="c9cdff13-0b23-44aa-a05a-d67f33830941")
    static bool held(Button button,uint32_t port) { return input.held(button,port); }
    EPOK_FUNCTION(BlueprintPure, Id="21eb4649-175d-4da4-85fc-b126f3025ed2")
    static bool pressed(Button button,uint32_t port) { return input.pressed(button,port); }
    EPOK_FUNCTION(BlueprintPure, Id="90a7de14-233d-47b2-8c88-e2aefc29fe91")
    static bool released(Button button,uint32_t port) { return input.released(button,port); }
    EPOK_FUNCTION(BlueprintPure, Id="9e3900d7-4ca1-4949-bd68-3eb225c47252")
    static bool frame_pressed(Button button,uint32_t port) { return input.frame_pressed(button,port); }
    EPOK_FUNCTION(BlueprintPure, Id="db95d98b-0dcf-4123-b01a-555a18f690c1")
    static bool frame_released(Button button,uint32_t port) { return input.frame_released(button,port); }
    EPOK_FUNCTION(BlueprintPure, Id="c1100ee5-d446-4f34-a65f-4f05dfce1f5c")
    static InputAxisSample axis(Axis axis,uint32_t port) {
        InputAxisSample result;
        result.connected=input.connected(port);
        result.analog=input.analog(port);
        if(result.analog&&uint32_t(axis)<4)
            result.value=Fixed(int32_t(input.axis_raw(axis,port)),Fixed::RAW);
        return result;
    }
};

struct EPOK_FUNCTION_LIBRARY(Category="Time", Id="5f34fef1-021f-4d96-85ad-427be29b84ab") TimeLibrary {
    EPOK_FUNCTION(BlueprintPure, Id="649d556b-cf71-4f56-8157-58123f71b32d")
    static TimeSnapshot snapshot() {
        return TimeSnapshot{
            time.paused(),time.ticks,time.dropped_steps,time.frame_microseconds,
            Fixed(time.delta_raw,Fixed::RAW)
        };
    }
    EPOK_FUNCTION(BlueprintPure, Id="0ff677d2-94a2-4fe7-a3d5-805a0ac8b086")
    static bool paused() { return time.paused(); }
    EPOK_FUNCTION(BlueprintCallable, Id="596f008d-3b03-41b2-a0ea-5dc278af6c15")
    static void set_paused(bool paused) { time.set_paused(paused); }
};

struct EPOK_FUNCTION_LIBRARY(Category="Math", Id="895f8595-ac0d-4584-b749-df23ae5dfbde") MathLibrary {
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="6b6ba8bd-8446-4fef-a147-e41b61f88a61") static Fixed clamp(Fixed value,Fixed minimum,Fixed maximum){return value<minimum?minimum:value>maximum?maximum:value;}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="53a63dd4-901c-4f40-92b3-241c879e96a7") static Fixed lerp(Fixed from,Fixed to,Fixed alpha){return from+(to-from)*clamp(alpha,0.0,1.0);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="380031af-9e10-4051-823b-7468b3a95f15") static Fixed smoothstep(Fixed alpha){alpha=clamp(alpha,0.0,1.0);return alpha*alpha*(Fixed(3.0)-Fixed(2.0)*alpha);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="4fe0f030-091c-4d55-984d-769077622c33") static GameplayVector3 vector3(Fixed x,Fixed y,Fixed z){return {x,y,z};}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="74678bdd-b229-46e9-949a-7256f1444183") static GameplayVector3 add(GameplayVector3 a,GameplayVector3 b){return {a.x+b.x,a.y+b.y,a.z+b.z};}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="d32131a5-2c47-43f2-8907-35e40389cfb5") static GameplayVector3 scale(GameplayVector3 value,Fixed amount){return {value.x*amount,value.y*amount,value.z*amount};}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="ab6fa418-8b8e-4c20-b41a-696c9cd61575") static GameplayVector3 subtract(GameplayVector3 a,GameplayVector3 b){return {a.x-b.x,a.y-b.y,a.z-b.z};}
    // Trigonometry, roots and angle arithmetic in the engine's own Q12 form.
    // Every one of these is integer-only, so a script reaches the same bits the
    // renderer and the collision solver already agree on, on host and on target.
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="eeee7d31-a8c6-437a-95db-10e163771b24") static Fixed sine_degrees(Fixed degrees){return sin_degrees(degrees);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="565552a8-869a-4ca7-a1d8-a7ad6e1508a7") static Fixed cosine_degrees(Fixed degrees){return cos_degrees(degrees);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="6977e1b9-1736-4c49-b003-2a7b3bdc98a2") static Fixed square_root(Fixed value){return Fixed(fixed_math::q_sqrt(value.raw()),Fixed::RAW);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="57b625c6-06d7-4635-b551-17a75ceffa3b") static Fixed length2(Fixed x,Fixed y){return Fixed(int32_t(fixed_math::sqrt64(uint64_t(int64_t(x.raw())*x.raw()+int64_t(y.raw())*y.raw()))),Fixed::RAW);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="625d5493-89ff-401f-a535-308206d5f4d4") static Fixed length(GameplayVector3 value){return Fixed(int32_t(fixed_math::sqrt64(uint64_t(int64_t(value.x.raw())*value.x.raw()+int64_t(value.y.raw())*value.y.raw()+int64_t(value.z.raw())*value.z.raw()))),Fixed::RAW);}
    // 0 <= result < 360. A turn is exact in Q12, so wrapping never drifts.
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="9b882fef-4a7b-4463-81aa-65b2f7e0dcd0") static Fixed wrap_degrees(Fixed degrees){const int32_t turn=360*4096;int32_t value=degrees.raw()%turn;if(value<0)value+=turn;return Fixed(value,Fixed::RAW);}
    // Shortest signed turn from one heading to another, in -180..180.
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="c5b4efca-6c21-42fb-b9bf-f354902d7ee4") static Fixed delta_degrees(Fixed from,Fixed to){const int32_t turn=360*4096;int32_t delta=(to.raw()-from.raw())%turn;if(delta>turn/2)delta-=turn;else if(delta<-turn/2)delta+=turn;return Fixed(delta,Fixed::RAW);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="3e369e64-cb38-4930-9842-02bbf054c5d9") static Fixed move_toward(Fixed from,Fixed to,Fixed max_step){const Fixed delta=to-from;if(delta>max_step)return from+max_step;if(delta<-max_step)return from-max_step;return to;}
    // The same approach on a circle: turns the short way and never overshoots.
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="bacfcfc1-e38c-44d2-b47d-19ec3ae37f8d") static Fixed move_toward_degrees(Fixed from,Fixed to,Fixed max_step){const Fixed delta=delta_degrees(from,to);if(delta>max_step)return from+max_step;if(delta<-max_step)return from-max_step;return to;}
    // Heading of a planar vector in degrees, measured from +y towards +x, so a
    // stick pushed forward reads 0 and a stick pushed right reads 90.
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="1057a007-0420-4b95-8f48-140be81f9ba5") static Fixed heading_degrees(Fixed x,Fixed y){
        const int32_t raw_x=x.raw(),raw_y=y.raw();
        const int32_t ax=raw_x<0?-raw_x:raw_x,ay=raw_y<0?-raw_y:raw_y;
        const int32_t large=ax>ay?ax:ay,small=ax>ay?ay:ax;
        if(!large)return Fixed(0,Fixed::RAW);
        const int32_t ratio=int32_t(int64_t(small)*4096/large);
        int32_t angle=int32_t(int64_t(ratio)*(45*4096+16*(4096-ratio))/4096);
        if(ax>ay)angle=90*4096-angle;
        if(raw_y<0)angle=180*4096-angle;
        if(raw_x<0)angle=360*4096-angle;
        return Fixed(angle,Fixed::RAW);
    }
    // The engine's standard analog dead zone, as a share of full deflection.
    // It is fixed rather than a parameter so every language reaches the same
    // raw threshold without depending on how a literal is rounded.
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="bf76ade4-3e06-44b8-b6af-baa40968ceaf") static Fixed stick_dead_zone(){return Fixed(491,Fixed::RAW);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="5b0a28df-53d9-40e4-b7ca-c5381f473771") static MovementIntent stick_intent(Fixed x,Fixed y){
        constexpr int32_t full=4096,dead_zone=491;
        int32_t raw_x=x.raw(),raw_y=y.raw();
        if(raw_x<-full)raw_x=-full;if(raw_x>full)raw_x=full;
        if(raw_y<-full)raw_y=-full;if(raw_y>full)raw_y=full;
        const int32_t magnitude=int32_t(fixed_math::sqrt64(uint64_t(int64_t(raw_x)*raw_x+int64_t(raw_y)*raw_y)));
        if(magnitude<=dead_zone)return {};
        const int32_t capped=magnitude>full?full:magnitude;
        const int32_t strength=(capped-dead_zone)*full/(full-dead_zone);
        MovementIntent result;
        result.x=Fixed(int32_t(int64_t(raw_x)*strength/magnitude),Fixed::RAW);
        result.y=Fixed(int32_t(int64_t(raw_y)*strength/magnitude),Fixed::RAW);
        result.strength=Fixed(strength,Fixed::RAW);
        return result;
    }
};

// The Vector3 half of the `utilities` group. It shares the category, and so the
// script namespace and the Blueprint palette, with epok::UtilityLibrary; it lives
// here only because GameplayVector3 is declared in this header. Like the scalar
// calls it is pure value: the caller owns the state and hands it back each frame.
struct EPOK_FUNCTION_LIBRARY(Category="Utilities", Id="07422ba3-cdbf-4384-bde6-19521f4e1c2c") UtilityVectorLibrary {
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="18823cc1-6141-40e6-8c2a-dc097bc32bfa") static GameplayVector3TweenState vector_tween_start(GameplayVector3 from,GameplayVector3 to,Fixed seconds,Ease easing){Tween timing;timing.start(0.0,1.0,seconds,easing);return gameplay_vector_tween(from,to,timing);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="5cd97b29-6570-4071-aede-8f095a2d59c6") static GameplayVector3TweenState vector_tween_schedule(GameplayVector3 from,GameplayVector3 to,Fixed seconds,Ease easing,Fixed delay_seconds,TweenLoop loop,uint32_t legs){Tween timing;timing.schedule(0.0,1.0,seconds,easing,delay_seconds,loop,legs);return gameplay_vector_tween(from,to,timing);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="ba4cbaf3-43a7-4476-aff8-c1cd48442c98") static Vector3TweenAdvanceSample vector_tween_advance(GameplayVector3TweenState state,Fixed delta_seconds){auto timing=gameplay_tween(state.timing);timing.advance(delta_seconds);Vector3TweenAdvanceSample result;result.state=gameplay_vector_tween(state.from,state.to,timing);result.completed=timing.completion_pending();result.value=gameplay_vector_tween_value(result.state);return result;}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="4e926f38-ba79-4cd8-9a64-aafd484d1c79") static GameplayVector3TweenState vector_tween_cancel(GameplayVector3TweenState state){auto timing=gameplay_tween(state.timing);timing.cancel();return gameplay_vector_tween(state.from,state.to,timing);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="9ffb6016-7a91-49ec-b8a6-315f6f5b91f0") static GameplayVector3 vector_tween_value(GameplayVector3TweenState state){return gameplay_vector_tween_value(state);}
};

struct EPOK_FUNCTION_LIBRARY(Category="World 2D", Id="624ed546-973a-40d1-8496-055eec580f18") World2DLibrary {
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="21b3ccb4-f31a-4e45-97f8-8df32e539768") static GameplayVector2 world_to_screen(GameplayCamera2D value,GameplayVector2 world){Camera2D camera;camera.position[0]=value.position.x;camera.position[1]=value.position.y;camera.zoom=value.zoom;camera.rotation=value.rotation;camera.viewport[0]=gameplay_i16(value.viewport_x);camera.viewport[1]=gameplay_i16(value.viewport_y);camera.viewport[2]=gameplay_i16(value.viewport_width);camera.viewport[3]=gameplay_i16(value.viewport_height);Fixed input[2]={world.x,world.y},output[2]={};epok::world_to_screen(camera,input,output);return {output[0],output[1]};}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="308085e2-43c4-473d-b225-24ef48aa4d1d") static GameplayVector2 screen_to_world(GameplayCamera2D value,GameplayVector2 screen){Camera2D camera;camera.position[0]=value.position.x;camera.position[1]=value.position.y;camera.zoom=value.zoom;camera.rotation=value.rotation;camera.viewport[0]=gameplay_i16(value.viewport_x);camera.viewport[1]=gameplay_i16(value.viewport_y);camera.viewport[2]=gameplay_i16(value.viewport_width);camera.viewport[3]=gameplay_i16(value.viewport_height);Fixed input[2]={screen.x,screen.y},output[2]={};epok::screen_to_world(camera,input,output);return {output[0],output[1]};}
};

struct EPOK_FUNCTION_LIBRARY(Category="Collision", Id="ae8a0a0d-d96e-4b15-bb0e-b4c9f87bbf47") CollisionLibrary {
    EPOK_FUNCTION(BlueprintCallable, Id="acebf3f3-9592-481f-820e-99e44b834f5d") static CollisionHitSample raycast_segment(GameplayVector3 origin,GameplayVector3 displacement,uint32_t mask,ObjectId ignore,bool triggers){
        Fixed a[3],b[3];gameplay_vector(origin,a);gameplay_vector(displacement,b);const auto hit=raycast(a,b,mask,gameplay_actor_data(ignore),triggers);CollisionHitSample result;result.hit=bool(hit);result.started_inside=hit.started_inside;result.actor=gameplay_actor_id(hit_entity(hit));result.fraction=hit.fraction;result.point=gameplay_vector(hit.point);result.normal=gameplay_vector(hit.normal);return result;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="757d2443-e77b-447f-b8cd-9d1c02cc2f61") static ObjectBatch8 overlap_box(GameplayAabb bounds,uint32_t mask,ObjectId ignore,bool triggers){
        Aabb box;gameplay_vector(bounds.minimum,box.min);gameplay_vector(bounds.maximum,box.max);DataHandle hits[8]{};ObjectBatch8 result;result.total=uint32_t(overlap(box,hits,8,mask,gameplay_actor_data(ignore),triggers));result.count=result.total<8?result.total:8;ObjectId* output[8]={&result.item0,&result.item1,&result.item2,&result.item3,&result.item4,&result.item5,&result.item6,&result.item7};for(uint32_t i=0;i<result.count;++i)*output[i]=gameplay_actor_id(hits[i]);return result;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="0d1768e0-c044-4fb4-953e-d2a3a9a44a85") static CollisionHitSample ground(ObjectId actor,Fixed distance,uint32_t mask){
        CollisionHitSample result;auto* data=gameplay_actor_data(actor);if(!data)return result;const auto hit=query_ground(*data,distance,mask);result.hit=bool(hit);result.started_inside=hit.started_inside;result.actor=gameplay_actor_id(hit_entity(hit));result.fraction=hit.fraction;result.point=gameplay_vector(hit.point);result.normal=gameplay_vector(hit.normal);return result;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="da298163-8f45-4b47-81c9-e2e70c89a566") static MoveSample move(ObjectId actor,GameplayVector3 displacement,uint32_t mask){
        MoveSample result;auto* data=gameplay_actor_data(actor);if(!data)return result;Fixed delta[3];gameplay_vector(displacement,delta);const auto moved=move_and_slide(*data,delta,mask);result.valid=true;result.grounded=moved.grounded;result.blocked=moved.blocked;result.unresolved_overlap=moved.unresolved_overlap;result.actor=moved.entity<0?ObjectId{}:gameplay_actor_id(DataHandle{uint16_t(moved.entity),moved.generation});result.displacement=gameplay_vector(moved.displacement);result.normal=gameplay_vector(moved.normal);return result;
    }
};

struct EPOK_FUNCTION_LIBRARY(Category="Scene", Id="e018f41e-b530-46f2-9331-9cf15f459fd2") SceneLibrary {
    EPOK_FUNCTION(BlueprintPure, Id="674276b6-f8a7-4030-9019-013f8cab971c") static SceneSnapshot snapshot(){return {uint32_t(current_scene()),scene_transition_count(),scene_rejected_count(),scene_loading(),scene_waiting()};}
    EPOK_FUNCTION(BlueprintPure, Id="f1d52fde-d755-4557-a8b4-dbed8c82af0e") static GameplayTransitionSnapshot transition_snapshot(){return {transition.phase,transition.audio_gain,transition.opacity,transition.busy(),transition.presented};}
    EPOK_FUNCTION(BlueprintCallable, Id="ea256570-baad-46f6-8966-1b9073c8bdf2") static bool request(uint32_t index){return request_scene(index);}
    EPOK_FUNCTION(BlueprintCallable, AsyncRequest, Id="43cc192e-6081-4ed4-a410-68c093d30d64") static bool request_with_transition(uint32_t index,GameplayTransitionOptions value){
#ifdef EPOK_TRANSITIONS
        TransitionOptions options;
        options.fade_out_ms=gameplay_u16(value.fade_out_ms);options.fade_in_ms=gameplay_u16(value.fade_in_ms);
        options.loading.color[0]=gameplay_u8(value.red);options.loading.color[1]=gameplay_u8(value.green);options.loading.color[2]=gameplay_u8(value.blue);
        return request_scene(index,options);
#else
        (void)value;return request_scene(index);
#endif
    }
    EPOK_FUNCTION(BlueprintPure, Id="5c70f7f6-3789-423e-83f1-608bc604bc2e") static FogSettings fog(){const auto& value=fog_environment;return {value.enabled,Fixed(value.start,Fixed::RAW),Fixed(value.end,Fixed::RAW),value.color[0],value.color[1],value.color[2]};}
    EPOK_FUNCTION(BlueprintCallable, Id="3de7b991-edcb-40e6-8739-ca4cc5e4039b") static bool set_fog(FogSettings value){
        // The editor validator accepts 0 <= start < end <= 128 with a span of at
        // least one Q12 step; an out-of-range range is rejected whole so a
        // scripted scene cannot diverge from an authored one.
        const int32_t start=value.start_distance.raw(),finish=value.end_distance.raw();
        if(start<0||finish>128*4096||finish-start<1)return false;
        fog_environment.enabled=value.enabled;fog_environment.start=start;fog_environment.end=finish;
        fog_environment.color[0]=gameplay_u8(value.red);fog_environment.color[1]=gameplay_u8(value.green);fog_environment.color[2]=gameplay_u8(value.blue);
        return true;
    }
    // The post-HUD fade to black, 0 clear to 255 opaque, clamped rather than
    // rejected because every amount above the range has one nearest valid value.
    // The getter reports the authored amount; what is drawn is the larger of it
    // and a running transition's opacity, which `transition_snapshot` already
    // reports. The amount survives scene activation, so a game can fade out,
    // request a scene and fade back in.
    EPOK_FUNCTION(BlueprintPure, Id="91c95c93-f719-430c-b8e0-1c5863468be9") static uint32_t screen_fade(){return epok::screen_fade;}
    EPOK_FUNCTION(BlueprintCallable, Id="2c441c7c-296e-4932-9b55-65a8f042a398") static void set_screen_fade(uint32_t value){
        // `epok::` is load-bearing on both accessors: this library is in namespace
        // epok, so an unqualified `screen_fade` in a member body finds the getter
        // above rather than the store it reads.
        epok::screen_fade=gameplay_u8(value);
    }
    EPOK_FUNCTION(BlueprintPure, Id="42291949-a4c5-454e-a5d6-09c5c3b1f698") static ObjectId active_camera_actor(){return gameplay_actor_id(active_camera());}
    EPOK_FUNCTION(BlueprintCallable, Id="e36ec48c-b2fe-49a2-8935-894d346882f7") static bool set_camera(ObjectId actor){return set_active_camera(gameplay_actor_data(actor));}
    EPOK_FUNCTION(BlueprintCallable, Id="b56f74e0-f7cf-4703-98d7-574a3d3387df") static ProjectedPoint project(GameplayVector3 world){Fixed input[3],screen[2]={};gameplay_vector(world,input);ProjectedPoint result;result.success=camera_project(input,screen);if(result.success){result.x=screen[0];result.y=screen[1];}return result;}
};

struct EPOK_FUNCTION_LIBRARY(Category="Resources", Id="ef57aa44-0ab3-41bd-b6d2-dc9f5f68f52e") ResourceLibrary {
    EPOK_FUNCTION(BlueprintPure, Id="42a503b9-9372-425d-8414-8c11c3818e49") static ResourceSnapshot snapshot(){const auto& r=resource_usage;return {r.alive_slots,r.active_slots,r.slot_capacity,r.scene_banks,r.textures,r.resident_texture_bytes,r.active_texture_bytes,r.particles,r.particle_peak,r.particle_dropped,r.mesh_triangles,r.sprite_triangles,r.dropped_primitives,r.frame_scanlines};}
    EPOK_FUNCTION(BlueprintPure, Id="e4656ba6-bc57-4202-a729-47da22d1b5e9") static SkeletalQuerySnapshot skeletal_queries(){const auto& value=skeletal_query_detail::stats();return {value.calls,value.vertices,value.bones,value.decoded_bytes,value.failures};}
    EPOK_FUNCTION(BlueprintCallable, Id="f3913f08-b2a5-4a43-ad5d-d711166b1703") static void clear_skeletal_queries(){skeletal_query_detail::stats()={};}
};

struct EPOK_FUNCTION_LIBRARY(Category="Playback", Id="d9276961-a79f-421e-8c89-41b02844c680") PlaybackLibrary {
    EPOK_FUNCTION(BlueprintCallable, Capability=timeline, Id="9853023f-f92a-4f36-9bbc-7a02b61e5a02") static timeline::Handle play_sequence(ObjectId component){return bp::api::play_sequence_component(component);}
    EPOK_FUNCTION(BlueprintCallable, Capability=timeline, Id="f3444ea5-610b-4d98-a847-bd086758e682") static bool stop_sequence(timeline::Handle handle){return bp::api::stop_sequence(handle);}
    EPOK_FUNCTION(BlueprintCallable, Capability=timeline, Id="6d5d12e5-fd8e-4187-b435-3735ff546d6f") static bool pause_sequence(timeline::Handle handle){return bp::api::pause_sequence(handle);}
    EPOK_FUNCTION(BlueprintCallable, Capability=timeline, Id="82e4534f-de22-4c01-96c9-68f25a093450") static bool resume_sequence(timeline::Handle handle){return bp::api::resume_sequence(handle);}
    EPOK_FUNCTION(BlueprintPure, Capability=timeline, Id="260e2851-f0a8-4a32-95a2-aad859974166") static GameplayPlaybackSnapshot sequence_state(timeline::Handle handle){bp::PlaybackWait wait;wait.begin(handle);const auto state=bp::api::playback_snapshot(wait);return {state.result,state.revision};}
    EPOK_FUNCTION(BlueprintCallable, Capability=effect, Id="c455542c-0362-405d-a16d-a80077fb5a78") static effects::Handle play_effect(ObjectId component){return bp::api::play_effect_component(component);}
    EPOK_FUNCTION(BlueprintCallable, Capability=effect, Id="b642218b-eb3c-4d6f-a6bd-c20123a83dc4") static bool stop_effect(effects::Handle handle){return bp::api::stop_effect(handle);}
    EPOK_FUNCTION(BlueprintCallable, Capability=effect, Id="f70cf390-d4bf-4cb9-990a-008f813c9522") static bool pause_effect(effects::Handle handle){return bp::api::pause_effect(handle);}
    EPOK_FUNCTION(BlueprintCallable, Capability=effect, Id="22fa1ff4-d2bb-43d9-98d4-013b809a74eb") static bool resume_effect(effects::Handle handle){return bp::api::resume_effect(handle);}
    EPOK_FUNCTION(BlueprintCallable, Capability=effect, Id="27c04535-3e5a-4133-8600-b11ee344f836") static bool burst_effect(effects::Handle handle,uint32_t count){return bp::api::burst_effect(handle,count);}
    EPOK_FUNCTION(BlueprintPure, Capability=effect, Id="a8d657bf-b554-40c9-ab20-f386292a2a4e") static GameplayPlaybackSnapshot effect_state(effects::Handle handle){bp::PlaybackWait wait;wait.begin(handle);const auto state=bp::api::playback_snapshot(wait);return {state.result,state.revision};}
    EPOK_FUNCTION(BlueprintPure, Capability=effect, Id="0990c907-4846-49c3-818b-4a0c47b450e7") static timeline::Handle effect_sequence(effects::Handle handle){return bp::api::effect_sequence(handle);}
};

struct EPOK_FUNCTION_LIBRARY(Category="Memory Card", Id="6073b812-ff1e-4764-9ba2-c77bac850e67") MemoryCardLibrary {
    EPOK_FUNCTION(BlueprintPure, Id="6534bc33-83f2-4a68-a575-1ed7175def0a") static MemoryCardSnapshot snapshot(){const auto& s=memory_card.status();return {s.state,s.operation,s.error,s.last_rejection,s.request,s.completed,s.payload_bytes,memory_card.file_count()};}
    EPOK_FUNCTION(BlueprintCallable, AsyncRequest, Id="19d34cc6-ac44-4c7a-a070-6bfe6d50505d") static bool probe(uint32_t port){return memory_card.probe(port);}
    EPOK_FUNCTION(BlueprintCallable, AsyncRequest, Id="711fb705-bfa1-468c-81c3-e3aca54a3f1c") static bool list(uint32_t port){return memory_card.list(port);}
    EPOK_FUNCTION(BlueprintCallable, AsyncRequest, Id="7c0c783d-c664-4a61-ac7b-ff62e77598a4") static bool read(uint32_t slot,uint32_t port){const char* name=gameplay_card_slot_name(slot);return name&&memory_card.read(name,port);}
    EPOK_FUNCTION(BlueprintCallable, AsyncRequest, Id="20c2c7eb-ecb9-487e-8448-68d26d059c09") static bool write(uint32_t slot,SavePayload8 payload,uint32_t port){const char* name=gameplay_card_slot_name(slot);return name&&memory_card.write(name,"EPOK Save",&payload,sizeof(payload),port);}
    EPOK_FUNCTION(BlueprintPure, Id="86ed6f6e-6591-4832-9eec-73944286c701") static SavePayload8 payload(){SavePayload8 value;if(memory_card.size()>=sizeof(value)){const uint8_t* source=memory_card.data();uint8_t* target=reinterpret_cast<uint8_t*>(&value);for(uint32_t i=0;i<sizeof(value);++i)target[i]=source[i];}return value;}
    EPOK_FUNCTION(BlueprintPure, Id="c53e0dba-d8f4-4e2d-b2bf-a17f40c336fd") static CardFileSample file(uint32_t index){CardFileSample result;if(index>=memory_card.file_count())return result;const auto& file=memory_card.files()[index];uint32_t hash=2166136261u;for(uint32_t i=0;i<20&&file.name[i];++i){hash^=uint8_t(file.name[i]);hash*=16777619u;}result.valid=true;result.name_hash=hash;result.blocks=file.blocks;return result;}
    EPOK_FUNCTION(BlueprintCallable, Id="c9819ed2-77b9-4a3d-801d-aa24d72f7ff2") static void clear_staged_payload(){auto* words=gameplay_save_words();for(uint32_t i=0;i<MemoryCardService::max_payload/4;++i)words[i]=0;}
    EPOK_FUNCTION(BlueprintCallable, Id="5acc07ae-8b3d-4f61-b7a7-a1ac96eb7788") static bool set_staged_word(uint32_t index,uint32_t value){if(index>=MemoryCardService::max_payload/4)return false;gameplay_save_words()[index]=value;return true;}
    EPOK_FUNCTION(BlueprintPure, Id="e2e0df92-13dd-4a46-99e5-54e255c8446b") static uint32_t staged_word(uint32_t index){return index<MemoryCardService::max_payload/4?gameplay_save_words()[index]:0;}
    EPOK_FUNCTION(BlueprintCallable, AsyncRequest, Id="c2809980-126a-4ed0-bf4f-1a073766d813") static bool write_staged(uint32_t slot,uint32_t bytes,uint32_t port){const char* name=gameplay_card_slot_name(slot);return name&&bytes<=MemoryCardService::max_payload&&memory_card.write(name,"EPOK Save",gameplay_save_words(),bytes,port);}
    EPOK_FUNCTION(BlueprintPure, Id="3164add9-daa1-4853-a44d-03b649f93f11") static uint32_t loaded_word(uint32_t index){const uint32_t offset=index*4;if(index>=MemoryCardService::max_payload/4||offset>=memory_card.size())return 0;uint32_t value=0;const auto* data=memory_card.data();for(uint32_t byte=0;byte<4&&offset+byte<memory_card.size();++byte)value|=uint32_t(data[offset+byte])<<(byte*8);return value;}
};

} // namespace epok
