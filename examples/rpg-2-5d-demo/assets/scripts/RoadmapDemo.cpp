#include "RoadmapDemo.hpp"
using namespace epok;
void RoadmapDemo::start(Transform&){temporary={};}
void RoadmapDemo::frame_update(Transform&,uint32_t){
    if(input.frame_pressed(Button::Start))time.set_paused(!time.paused());
}
void RoadmapDemo::update(Transform& transform,Fixed dt){
    Fixed movement[3]={0.0,-0.1,0.0};
    if(input.held(Button::Left))movement[0]-=dt*2;
    if(input.held(Button::Right))movement[0]+=dt*2;
    if(input.held(Button::Up))movement[2]+=dt*2;
    if(input.held(Button::Down))movement[2]-=dt*2;
    auto hit=move_and_slide(entity(),movement);
    entity().sprite.flip_x=movement[0].raw()<0;
    if(input.pressed(Button::Cross))if(auto* emitter=find_entity("Sparks")){
        emitter->transform.position[0]=transform.position[0];emitter->transform.position[1]=transform.position[1]+Fixed(0.3);emitter->transform.position[2]=transform.position[2];emitter->particle_emitter.burst();
    }
    if(input.pressed(Button::Circle)){
        if(auto* old=temporary.get())destroy_entity(old);
        else if(auto* spawned=create_entity("Pooled sprite")){
            spawned->sprite=entity().sprite;spawned->sprite.color[0]=100;spawned->sprite.unlit=true;
            spawned->transform.position[0]=transform.position[0]+Fixed(0.8);spawned->transform.position[1]=transform.position[1];spawned->transform.position[2]=transform.position[2];temporary=handle(spawned);
        }
    }
    if(input.pressed(Button::Select))request_scene(size_t(current_scene()==0?1:0));
    if(input.pressed(Button::Triangle))if(auto* effect=find_entity("Campfire"))set_active(effect,!is_active(effect));
    if(input.pressed(Button::Square)){
        auto ground=query_ground(entity(),2.0);auto* status=find_entity("Status");
        if(status)status->text.set_text(ground?"Ground found. Collision ready.":"No ground beneath the character.");
    }
    (void)hit;
}
void RoadmapDemo::on_trigger(EntityHandle other,TriggerPhase phase){
    if(!other.get())return;
    if(auto* status=find_entity("Status")){
        if(phase==TriggerPhase::Enter)status->text.set_text("Portal entered. SELECT: switch scene.");
        if(phase==TriggerPhase::Exit)status->text.set_text("Portal exited. References valid.");
    }
}
