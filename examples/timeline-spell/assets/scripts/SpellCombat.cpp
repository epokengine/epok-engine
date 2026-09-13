#include "SpellCombat.hpp"
#include "psyqo/xprintf.h"

void SpellTarget::take_damage(uint32_t amount){
    if(!health||!amount)return;
    health=amount>=health?0:health-amount;
    if(hits!=UINT32_MAX)++hits;
    if(auto* label=status.get()){
        char message[80];
        snprintf(message,sizeof(message),"Target HP: %u / 100    Impact hits: %u",unsigned(health),unsigned(hits));
        label->text.set_text(message);
    }
    entity().sprite.color[0]=255;
    entity().sprite.color[1]=health?128:48;
    entity().sprite.color[2]=health?96:48;
}
void SpellCombat::remember_effect(epok::effects::Handle value){
    playback=value;busy=true;
    if(auto* label=status.get())label->text.set_text("Fireball active. Damage occurs at impact.");
}
void SpellCombat::finished(){
    busy=false;
    if(auto* label=status.get())label->text.set_text("Fireball complete. Particle tail has drained.");
}
void SpellCombat::cancelled(){
    busy=false;
    if(auto* label=status.get())label->text.set_text("Fireball stopped. No further damage will be applied.");
}
void SpellCombat::start(epok::Transform&){cast_spell(25,target);}
void SpellCombat::update(epok::Transform&,epok::Fixed){
    if(epok::input.pressed(epok::Button::Cross)&&!busy)cast_spell(25,target);
    if(epok::input.pressed(epok::Button::Circle)&&busy)cancel_spell();
    if(epok::input.pressed(epok::Button::Square))epok::request_scene("Combat");
}
void SpellCombat::frame_update(epok::Transform&,uint32_t){
    if(epok::input.frame_pressed(epok::Button::Start))epok::time.set_paused(!epok::time.paused());
}
