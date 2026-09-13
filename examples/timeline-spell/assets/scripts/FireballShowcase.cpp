#include "FireballShowcase.hpp"
#include "particle_effect_service.hpp"

void FireballShowcase::update(epok::Transform&,epok::Fixed dt){
    using namespace epok;
    const auto owner=handle(&entity());
    auto* component=effects::component(owner);
    if(!component)return;
    const auto state=effects::pool.state(component->playback);
    if(state==effects::State::Playing||state==effects::State::Draining){delay=0.0;return;}
    delay+=dt;
    if(delay>=Fixed(0.5)){effects::play(owner);delay=0.0;}
}
