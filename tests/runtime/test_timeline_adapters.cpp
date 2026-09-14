#include "TimelineAdapters.hpp"
#include <cassert>
#include <cstdio>

namespace epok {
unsigned adapter_audio_plays=0,adapter_audio_stops=0,adapter_camera_activations=0;
void AudioSource::play(){++adapter_audio_plays;}
void AudioSource::stop(){++adapter_audio_stops;}
bool AudioSource::is_playing()const{return false;}
bool set_active_camera(ActorData* value){if(!value||!value->camera)return false;++adapter_camera_activations;return true;}
}

#define EPOK_TEST_AUDIO_IMPLEMENTED
#include "actor_scene_fixture.hpp"
template<class T>struct Attached:T {void attach(epok::Actor& owner){this->m_owner=owner.id();}};
int main(){
    using namespace epok;
    test_reset_scene();auto& target=entities[0];
    auto* owner=test_registry.resolve<Actor>(test_owner(0));assert(owner);
    target.camera=true;
    target.camera_settings.field_of_view=107.0;
    target.transform.position[0]=13.0;
    target.transform.rotation[1]=24.0;
    Attached<TimelineCamera> camera;
    camera.attach(*owner);
    // Capture live component state without applying unrelated proxy defaults.
    constexpr uint64_t fov=UINT64_C(15657026159422320539);
    constexpr uint64_t position=UINT64_C(5859010025834633935);
    camera.timeline_sync(fov,true);
    assert(camera.field_of_view.raw()==107*4096);
    camera.field_of_view=65.0;
    camera.timeline_sync(fov,false);
    assert(target.camera_settings.field_of_view.raw()==65*4096);
    assert(target.transform.position[0].raw()==13*4096);
    camera.timeline_sync(position,true);
    assert(camera.position[0].raw()==13*4096);
    camera.position[0]=7.0;
    camera.timeline_sync(position,false);
    assert(target.transform.position[0].raw()==7*4096);
    assert(target.transform.rotation[1].raw()==24*4096);
    camera.field_of_view=107.0;
    camera.timeline_sync(fov,false);
    assert(target.camera_settings.field_of_view.raw()==107*4096);
    camera.field_of_view=Fixed(INT32_MAX,Fixed::RAW);
    camera.timeline_sync(fov,false);
    assert(target.camera_settings.field_of_view.raw()==120*4096);
    camera.activate();
    assert(adapter_camera_activations==1);
    target.camera=false;
    camera.field_of_view=30.0;
    camera.timeline_sync(fov,false);
    camera.activate();
    assert(target.camera_settings.field_of_view.raw()==120*4096&&adapter_camera_activations==1);
    // A Transform-typed binding may still animate a non-camera's transform.
    camera.position[0]=9.0;
    camera.timeline_sync(position,false);
    assert(target.transform.position[0].raw()==9*4096);
    camera.timeline_sync(UINT64_MAX,false);
    assert(target.transform.position[0].raw()==9*4096);

    Attached<TimelineAudio> audio;audio.attach(*owner);
    audio.play();audio.stop();
    assert(adapter_audio_plays==0&&adapter_audio_stops==0);
    target.audio.enabled=true;
    audio.play();audio.stop();
    assert(adapter_audio_plays==1&&adapter_audio_stops==1);
    Attached<TimelineEmitter> emitter;emitter.attach(*owner);
    target.particle_emitter.enabled=true;
    // Restoring an existing valid upper bound must not narrow the component's
    // authoring range. Particle rate supports 512; queue capacity is separate.
    constexpr uint64_t rate=UINT64_C(6300725732659222235);
    target.particle_emitter.rate=512.0;
    emitter.timeline_sync(rate,true);
    target.particle_emitter.rate=0.0;
    emitter.timeline_sync(rate,false);
    assert(target.particle_emitter.rate.raw()==512*4096);
    constexpr uint64_t lifetime=UINT64_C(10671107849971937109);
    emitter.lifetime=Fixed(INT32_MIN,Fixed::RAW);
    emitter.timeline_sync(lifetime,false);
    assert(target.particle_emitter.lifetime.raw()==68);
    emitter.stop();assert(!target.particle_emitter.playing);
    emitter.play();assert(target.particle_emitter.playing);
    emitter.burst(0);assert(target.particle_emitter.pending==0);
    emitter.burst(300);
    assert(target.particle_emitter.pending==256&&particle_stats.dropped==44);
    emitter.burst(UINT32_MAX);
    assert(target.particle_emitter.pending==256&&particle_stats.dropped==UINT32_MAX);
    target.particle_emitter.enabled=false;
    target.particle_emitter.pending=0;
    emitter.burst(300);assert(target.particle_emitter.pending==0);
    emitter.stop();assert(target.particle_emitter.playing);
    Attached<TimelinePalette> palette;palette.attach(*owner);
    target.palette_animator.enabled=true;
    target.palette_animator.remainder=25;target.palette_animator.offset=7;
    palette.reset();
    assert(target.palette_animator.remainder==0&&target.palette_animator.offset==0);
    Attached<TimelineRect> rect;rect.attach(*test_ui_actor(0));target.rect.enabled=true;
    target.rect.size[0]=-1024.0;target.rect.size[1]=1024.0;
    constexpr uint64_t size=UINT64_C(7984496657147586794);
    rect.timeline_sync(size,true);
    target.rect.size[0]=0.0;target.rect.size[1]=0.0;
    rect.timeline_sync(size,false);
    assert(target.rect.size[0].raw()==-1024*4096&&target.rect.size[1].raw()==1024*4096);
    // Raw Q12 color conversion is deterministic and restores every authored byte.
    for(unsigned i=0;i<256;++i){
        const auto value=epok_timeline_adapter_detail::color(uint8_t(i));
        assert(epok_timeline_adapter_detail::color(value)==i);
    }
    assert(epok_timeline_adapter_detail::color(Fixed(INT32_MIN,Fixed::RAW))==0);
    assert(epok_timeline_adapter_detail::color(Fixed(INT32_MAX,Fixed::RAW))==255);
    std::puts("Timeline adapters: live capture, immediate writes/restoration, inherited transform, component guards and Q12 color round trips pass.");
}
