#pragma once
#include "epok.hpp"

// Epok Timeline component adapters, source version 1.
// Install into assets/scripts, then attach a class or derive a Blueprint from it.
// A track captures the live component at play; inspector proxy defaults never
// overwrite a component. Only the addressed property is synchronized, immediately
// before events and during restoration. All transforms are local to the parent.
// Keep these explicit IDs when renaming or extending an installed class.
namespace epok_timeline_adapter_detail {
inline epok::Fixed bounded(epok::Fixed value,int32_t low,int32_t high) {
    const auto raw=value.raw();
    return epok::Fixed(raw<low?low:raw>high?high:raw,epok::Fixed::RAW);
}
inline epok::Fixed color(uint8_t value) {
    return epok::Fixed((int32_t(value)*4096+127)/255,epok::Fixed::RAW);
}
inline uint8_t color(epok::Fixed value) {
    const auto raw=bounded(value,0,4096).raw();
    return uint8_t((raw*255+2048)/4096);
}
}

class EPOK_CLASS(Blueprintable,Id="313dd749-e96c-5caf-9aa2-47ee3486b814") TimelineTransform:public epok::Behaviour {
public:
    EPOK_PROPERTY(TimelineAnimatable,Id="21635438-8736-5c5a-9089-2a10c37656d6") epok::Fixed position[3]={0.0,0.0,0.0};
    EPOK_PROPERTY(TimelineAnimatable,Id="7aa29452-edc6-5f08-813f-813961e03e21") epok::Fixed rotation[3]={0.0,0.0,0.0};
    EPOK_PROPERTY(TimelineAnimatable,Id="e6a825ce-96e4-5d74-a2d2-13b4e235cdfd") epok::Fixed scale[3]={1.0,1.0,1.0};
    void update(epok::Transform&,epok::Fixed)override{}
    void timeline_sync(uint64_t property,bool read)override {
        switch(property) {
        case UINT64_C(5859010025834633935): // 21635438-8736-5c5a-9089-2a10c37656d6
            if(read)position[0]=entity().transform.position[0];else entity().transform.position[0]=position[0];
            if(read)position[1]=entity().transform.position[1];else entity().transform.position[1]=position[1];
            if(read)position[2]=entity().transform.position[2];else entity().transform.position[2]=position[2];
            return;
        case UINT64_C(13780169784377536898): // 7aa29452-edc6-5f08-813f-813961e03e21
            if(read)rotation[0]=entity().transform.rotation[0];else entity().transform.rotation[0]=rotation[0];
            if(read)rotation[1]=entity().transform.rotation[1];else entity().transform.rotation[1]=rotation[1];
            if(read)rotation[2]=entity().transform.rotation[2];else entity().transform.rotation[2]=rotation[2];
            return;
        case UINT64_C(15940121614410949054): // e6a825ce-96e4-5d74-a2d2-13b4e235cdfd
            if(read)scale[0]=entity().transform.scale[0];else entity().transform.scale[0]=scale[0];
            if(read)scale[1]=entity().transform.scale[1];else entity().transform.scale[1]=scale[1];
            if(read)scale[2]=entity().transform.scale[2];else entity().transform.scale[2]=scale[2];
            return;
        default:epok::Behaviour::timeline_sync(property,read);return;
        }
    }
};

class EPOK_CLASS(Blueprintable,Id="b24d1abd-e72f-5d3d-9f30-85bcb20c3187",TimelineRequires=Camera) TimelineCamera:public TimelineTransform {
public:
    EPOK_PROPERTY(TimelineAnimatable,Id="96829145-5fad-5d63-b350-a4400d855544") epok::Fixed field_of_view=90;
    EPOK_FUNCTION(TimelineAction,Id="6f9f18ca-c171-5f8d-8a21-0d2b0593c2a2") void activate(){if(entity().camera)epok::set_active_camera(&entity());}
    void timeline_sync(uint64_t property,bool read)override {
        switch(property) {
        case UINT64_C(15657026159422320539): // 96829145-5fad-5d63-b350-a4400d855544
            if(!entity().camera)return;
            if(read)field_of_view=entity().camera_settings.field_of_view;else entity().camera_settings.field_of_view=epok_timeline_adapter_detail::bounded(field_of_view,102400,491520);
            return;
        default:TimelineTransform::timeline_sync(property,read);return;
        }
    }
};

class EPOK_CLASS(Blueprintable,Id="763addbb-1525-571e-abda-303981fee86e",TimelineRequires=AudioSource) TimelineAudio:public epok::Behaviour {
public:
    EPOK_PROPERTY(TimelineAnimatable,Id="045b5693-10f1-5844-bd85-7c3b949cf220") epok::Fixed volume=1;
    EPOK_PROPERTY(TimelineAnimatable,Id="3f22001d-a375-5767-acd3-910c3f993196") epok::Fixed pitch=1;
    EPOK_FUNCTION(TimelineCallable,Id="2867cbf2-8e41-52a3-bdb2-16ef3f12ee23") void play(){if(entity().audio.enabled)entity().audio.play();}
    EPOK_FUNCTION(TimelineAction,Id="5938eccf-9c36-54ee-8549-c9bf39a41654") void stop(){if(entity().audio.enabled)entity().audio.stop();}
    void update(epok::Transform&,epok::Fixed)override{}
    void timeline_sync(uint64_t property,bool read)override {
        switch(property) {
        case UINT64_C(1259870957092764419): // 045b5693-10f1-5844-bd85-7c3b949cf220
            if(!entity().audio.enabled)return;
            if(read)volume=entity().audio.volume;else entity().audio.volume=epok_timeline_adapter_detail::bounded(volume,0,4096);
            return;
        case UINT64_C(16871135828429460167): // 3f22001d-a375-5767-acd3-910c3f993196
            if(!entity().audio.enabled)return;
            if(read)pitch=entity().audio.pitch;else entity().audio.pitch=epok_timeline_adapter_detail::bounded(pitch,1024,16384);
            return;
        default:epok::Behaviour::timeline_sync(property,read);return;
        }
    }
};

class EPOK_CLASS(Blueprintable,Id="14dda09f-4678-5c86-85a3-97a48f07b7dd",TimelineRequires=Light) TimelineLight:public TimelineTransform {
public:
    EPOK_PROPERTY(TimelineAnimatable,Id="687cddb2-1839-53d7-a60e-d4038fdf36f0") epok::Fixed intensity=0.8;
    EPOK_PROPERTY(TimelineAnimatable,Id="ac2462a3-8034-5a57-ab26-83339ab01662") epok::Fixed range=8;
    EPOK_PROPERTY(TimelineAnimatable,Id="27f981b7-c9da-537a-bda5-c0dec6cff1dd") epok::Fixed color[3]={1.0,1.0,1.0};
    void timeline_sync(uint64_t property,bool read)override {
        switch(property) {
        case UINT64_C(14329296674651517980): // 687cddb2-1839-53d7-a60e-d4038fdf36f0
            if(!entity().light.enabled)return;
            if(read)intensity=entity().light.intensity;else entity().light.intensity=epok_timeline_adapter_detail::bounded(intensity,0,8192);
            return;
        case UINT64_C(7633748854292457688): // ac2462a3-8034-5a57-ab26-83339ab01662
            if(!entity().light.enabled)return;
            if(read)range=entity().light.range;else entity().light.range=epok_timeline_adapter_detail::bounded(range,41,524288);
            return;
        case UINT64_C(10274688836752898587): // 27f981b7-c9da-537a-bda5-c0dec6cff1dd
            if(!entity().light.enabled)return;
            if(read)color[0]=epok_timeline_adapter_detail::color(entity().light.color[0]);else entity().light.color[0]=epok_timeline_adapter_detail::color(color[0]);
            if(read)color[1]=epok_timeline_adapter_detail::color(entity().light.color[1]);else entity().light.color[1]=epok_timeline_adapter_detail::color(color[1]);
            if(read)color[2]=epok_timeline_adapter_detail::color(entity().light.color[2]);else entity().light.color[2]=epok_timeline_adapter_detail::color(color[2]);
            return;
        default:TimelineTransform::timeline_sync(property,read);return;
        }
    }
};

class EPOK_CLASS(Blueprintable,Id="2a0c0d28-acb9-56f9-ab4e-cb51e5ac1743",TimelineRequires=PaletteAnimator) TimelinePalette:public epok::Behaviour {
public:
    EPOK_PROPERTY(TimelineAnimatable,Id="7daa39c1-993b-5a2d-9aa7-7240da022dbe") epok::Fixed speed=8;
    EPOK_PROPERTY(TimelineAnimatable,Id="9eb5bb70-90a6-5806-83b4-eb3e0a63209c") bool reverse=false;
    EPOK_FUNCTION(TimelineCallable,Id="e1965d05-410b-5fef-86b8-0e1ce038ce9a") void reset(){if(entity().palette_animator.enabled)entity().palette_animator.reset();}
    void update(epok::Transform&,epok::Fixed)override{}
    void timeline_sync(uint64_t property,bool read)override {
        switch(property) {
        case UINT64_C(8627092943292772508): // 7daa39c1-993b-5a2d-9aa7-7240da022dbe
            if(!entity().palette_animator.enabled)return;
            if(read)speed=entity().palette_animator.speed;else entity().palette_animator.speed=epok_timeline_adapter_detail::bounded(speed,0,245760);
            return;
        case UINT64_C(17994863362683766101): // 9eb5bb70-90a6-5806-83b4-eb3e0a63209c
            if(!entity().palette_animator.enabled)return;
            if(read)reverse=entity().palette_animator.reverse;else entity().palette_animator.reverse=reverse;
            return;
        default:epok::Behaviour::timeline_sync(property,read);return;
        }
    }
};

class EPOK_CLASS(Blueprintable,Id="77ab0039-7003-5514-a791-9f7a43ca4d59",TimelineRequires=ParticleEmitter) TimelineEmitter:public TimelineTransform {
public:
    EPOK_PROPERTY(TimelineAnimatable,Id="af42bcfa-f399-52c7-875a-f5c4a2867dec") epok::Fixed rate=8;
    EPOK_PROPERTY(TimelineAnimatable,Id="30a7f981-9454-5125-a56a-b729224507e2") epok::Fixed lifetime=1;
    EPOK_PROPERTY(TimelineAnimatable,Id="d8cf4aa4-410e-5d1c-97e3-0d23b0b71820") epok::Fixed velocity[3]={0.0,1.0,0.0};
    EPOK_PROPERTY(TimelineAnimatable,Id="12de302d-ee45-596b-9cbe-f14adea560ab") epok::Fixed spread[3]={0.3,0.2,0.3};
    EPOK_PROPERTY(TimelineAnimatable,Id="724178c2-fdf7-5f75-8c8a-5ea225e517c2") epok::Fixed gravity[3]={0.0,-0.2,0.0};
    EPOK_PROPERTY(TimelineAnimatable,Id="9e8ddc84-daa0-5f33-a063-6a5fc3590af3") epok::Fixed start_size=0.25;
    EPOK_PROPERTY(TimelineAnimatable,Id="b9233d0c-dff2-5cbf-a3c4-69998e8ae1a0") epok::Fixed end_size=0.05;
    EPOK_FUNCTION(TimelineCallable,Id="f9e59bb8-6132-509f-b6b7-619a7a1f10ae") void play(){if(entity().particle_emitter.enabled)entity().particle_emitter.play();}
    EPOK_FUNCTION(TimelineAction,Id="7dfe7279-2239-5417-aa52-fb8b8147d9f1") void stop(){if(entity().particle_emitter.enabled)entity().particle_emitter.stop();}
    EPOK_FUNCTION(TimelineCallable,Id="1007cad3-6710-5ac3-b3bc-b3435291a66c") void burst(uint32_t count){
        if(!entity().particle_emitter.enabled)return;
        auto& emitter=entity().particle_emitter;
        const uint32_t room=256-emitter.pending,accepted=count<room?count:room;
        emitter.pending+=uint16_t(accepted);
        const uint32_t dropped=count-accepted;
        auto& counter=epok::particle_stats.dropped;
        counter=UINT32_MAX-counter<dropped?UINT32_MAX:counter+dropped;
    }
    void timeline_sync(uint64_t property,bool read)override {
        switch(property) {
        case UINT64_C(6300725732659222235): // af42bcfa-f399-52c7-875a-f5c4a2867dec
            if(!entity().particle_emitter.enabled)return;
            if(read)rate=entity().particle_emitter.rate;else entity().particle_emitter.rate=epok_timeline_adapter_detail::bounded(rate,0,2097152);
            return;
        case UINT64_C(10671107849971937109): // 30a7f981-9454-5125-a56a-b729224507e2
            if(!entity().particle_emitter.enabled)return;
            if(read)lifetime=entity().particle_emitter.lifetime;else entity().particle_emitter.lifetime=epok_timeline_adapter_detail::bounded(lifetime,68,245760);
            return;
        case UINT64_C(1941359954913885089): // d8cf4aa4-410e-5d1c-97e3-0d23b0b71820
            if(!entity().particle_emitter.enabled)return;
            if(read)velocity[0]=entity().particle_emitter.velocity[0];else entity().particle_emitter.velocity[0]=epok_timeline_adapter_detail::bounded(velocity[0],-524288,524288);
            if(read)velocity[1]=entity().particle_emitter.velocity[1];else entity().particle_emitter.velocity[1]=epok_timeline_adapter_detail::bounded(velocity[1],-524288,524288);
            if(read)velocity[2]=entity().particle_emitter.velocity[2];else entity().particle_emitter.velocity[2]=epok_timeline_adapter_detail::bounded(velocity[2],-524288,524288);
            return;
        case UINT64_C(14426640552543062730): // 12de302d-ee45-596b-9cbe-f14adea560ab
            if(!entity().particle_emitter.enabled)return;
            if(read)spread[0]=entity().particle_emitter.spread[0];else entity().particle_emitter.spread[0]=epok_timeline_adapter_detail::bounded(spread[0],0,524288);
            if(read)spread[1]=entity().particle_emitter.spread[1];else entity().particle_emitter.spread[1]=epok_timeline_adapter_detail::bounded(spread[1],0,524288);
            if(read)spread[2]=entity().particle_emitter.spread[2];else entity().particle_emitter.spread[2]=epok_timeline_adapter_detail::bounded(spread[2],0,524288);
            return;
        case UINT64_C(10546923376391738746): // 724178c2-fdf7-5f75-8c8a-5ea225e517c2
            if(!entity().particle_emitter.enabled)return;
            if(read)gravity[0]=entity().particle_emitter.gravity[0];else entity().particle_emitter.gravity[0]=epok_timeline_adapter_detail::bounded(gravity[0],-524288,524288);
            if(read)gravity[1]=entity().particle_emitter.gravity[1];else entity().particle_emitter.gravity[1]=epok_timeline_adapter_detail::bounded(gravity[1],-524288,524288);
            if(read)gravity[2]=entity().particle_emitter.gravity[2];else entity().particle_emitter.gravity[2]=epok_timeline_adapter_detail::bounded(gravity[2],-524288,524288);
            return;
        case UINT64_C(13317457305208282192): // 9e8ddc84-daa0-5f33-a063-6a5fc3590af3
            if(!entity().particle_emitter.enabled)return;
            if(read)start_size=entity().particle_emitter.start_size;else entity().particle_emitter.start_size=epok_timeline_adapter_detail::bounded(start_size,0,131072);
            return;
        case UINT64_C(7058651935412449454): // b9233d0c-dff2-5cbf-a3c4-69998e8ae1a0
            if(!entity().particle_emitter.enabled)return;
            if(read)end_size=entity().particle_emitter.end_size;else entity().particle_emitter.end_size=epok_timeline_adapter_detail::bounded(end_size,0,131072);
            return;
        default:TimelineTransform::timeline_sync(property,read);return;
        }
    }
};

class EPOK_CLASS(Blueprintable,Id="465437e9-0cb6-549d-9599-bd6e1dfa50ca",TimelineRequires=RectTransform) TimelineRect:public epok::Behaviour {
public:
    EPOK_PROPERTY(TimelineAnimatable,Id="0c260655-c958-53ed-b60e-d5476751d07e") epok::Fixed position[2]={0.0,0.0};
    EPOK_PROPERTY(TimelineAnimatable,Id="1d62264b-3aec-584b-9a4a-5fce6ca10937") epok::Fixed size[2]={100.0,32.0};
    void update(epok::Transform&,epok::Fixed)override{}
    void timeline_sync(uint64_t property,bool read)override {
        switch(property) {
        case UINT64_C(2659875927787549232): // 0c260655-c958-53ed-b60e-d5476751d07e
            if(!entity().rect.enabled)return;
            if(read)position[0]=entity().rect.position[0];else entity().rect.position[0]=epok_timeline_adapter_detail::bounded(position[0],-4194304,4194304);
            if(read)position[1]=entity().rect.position[1];else entity().rect.position[1]=epok_timeline_adapter_detail::bounded(position[1],-4194304,4194304);
            return;
        case UINT64_C(7984496657147586794): // 1d62264b-3aec-584b-9a4a-5fce6ca10937
            if(!entity().rect.enabled)return;
            if(read)size[0]=entity().rect.size[0];else entity().rect.size[0]=epok_timeline_adapter_detail::bounded(size[0],-4194304,4194304);
            if(read)size[1]=entity().rect.size[1];else entity().rect.size[1]=epok_timeline_adapter_detail::bounded(size[1],-4194304,4194304);
            return;
        default:epok::Behaviour::timeline_sync(property,read);return;
        }
    }
};

class EPOK_CLASS(Blueprintable,Id="1ecf718f-bb79-56f8-9a39-53c52cb67a41",TimelineRequires=Text) TimelineText:public TimelineRect {
public:
    EPOK_PROPERTY(TimelineAnimatable,Id="89cb1c33-f41f-5858-befc-bb9faff7945f") epok::Fixed color[3]={1.0,1.0,1.0};
    EPOK_PROPERTY(TimelineAnimatable,Id="8cba8e89-5867-51e3-bbb8-68f2bfa5ef38") bool wrap=true;
    void timeline_sync(uint64_t property,bool read)override {
        switch(property) {
        case UINT64_C(312067917156951222): // 89cb1c33-f41f-5858-befc-bb9faff7945f
            if(!entity().text.enabled)return;
            if(read)color[0]=epok_timeline_adapter_detail::color(entity().text.color[0]);else entity().text.color[0]=epok_timeline_adapter_detail::color(color[0]);
            if(read)color[1]=epok_timeline_adapter_detail::color(entity().text.color[1]);else entity().text.color[1]=epok_timeline_adapter_detail::color(color[1]);
            if(read)color[2]=epok_timeline_adapter_detail::color(entity().text.color[2]);else entity().text.color[2]=epok_timeline_adapter_detail::color(color[2]);
            return;
        case UINT64_C(11099417250759729775): // 8cba8e89-5867-51e3-bbb8-68f2bfa5ef38
            if(!entity().text.enabled)return;
            if(read)wrap=entity().text.wrap;else entity().text.wrap=wrap;
            return;
        default:TimelineRect::timeline_sync(property,read);return;
        }
    }
};

class EPOK_CLASS(Blueprintable,Id="eb268d1d-92d3-5c5a-9544-11c13ee964dd",TimelineRequires=Image) TimelineImage:public TimelineRect {
public:
    EPOK_PROPERTY(TimelineAnimatable,Id="d097b075-c002-5fad-8670-c31709ed93a4") epok::Fixed color[3]={0.2,0.4,0.65};
    void timeline_sync(uint64_t property,bool read)override {
        switch(property) {
        case UINT64_C(5653150930357933886): // d097b075-c002-5fad-8670-c31709ed93a4
            if(!entity().image.enabled)return;
            if(read)color[0]=epok_timeline_adapter_detail::color(entity().image.color[0]);else entity().image.color[0]=epok_timeline_adapter_detail::color(color[0]);
            if(read)color[1]=epok_timeline_adapter_detail::color(entity().image.color[1]);else entity().image.color[1]=epok_timeline_adapter_detail::color(color[1]);
            if(read)color[2]=epok_timeline_adapter_detail::color(entity().image.color[2]);else entity().image.color[2]=epok_timeline_adapter_detail::color(color[2]);
            return;
        default:TimelineRect::timeline_sync(property,read);return;
        }
    }
};

class EPOK_CLASS(Blueprintable,Id="d90ba923-f04a-54cf-ab5b-e6dced35eebd",TimelineRequires=ProgressBar) TimelineProgress:public TimelineRect {
public:
    EPOK_PROPERTY(TimelineAnimatable,Id="c0542ee5-b823-5530-b013-7c16eedbece0") epok::Fixed value=0.75;
    EPOK_PROPERTY(TimelineAnimatable,Id="c224ae53-69eb-5e7b-973a-c5641f947c51") epok::Fixed color[3]={0.25,0.85,0.3};
    EPOK_PROPERTY(TimelineAnimatable,Id="359efbca-cf5d-5954-a1d2-b0fde40d7206") epok::Fixed background[3]={0.12,0.12,0.12};
    void timeline_sync(uint64_t property,bool read)override {
        switch(property) {
        case UINT64_C(14821324979561895513): // c0542ee5-b823-5530-b013-7c16eedbece0
            if(!entity().progress.enabled)return;
            if(read)value=entity().progress.value;else entity().progress.value=epok_timeline_adapter_detail::bounded(value,0,4096);
            return;
        case UINT64_C(13828947234099185252): // c224ae53-69eb-5e7b-973a-c5641f947c51
            if(!entity().progress.enabled)return;
            if(read)color[0]=epok_timeline_adapter_detail::color(entity().progress.color[0]);else entity().progress.color[0]=epok_timeline_adapter_detail::color(color[0]);
            if(read)color[1]=epok_timeline_adapter_detail::color(entity().progress.color[1]);else entity().progress.color[1]=epok_timeline_adapter_detail::color(color[1]);
            if(read)color[2]=epok_timeline_adapter_detail::color(entity().progress.color[2]);else entity().progress.color[2]=epok_timeline_adapter_detail::color(color[2]);
            return;
        case UINT64_C(342464961878183840): // 359efbca-cf5d-5954-a1d2-b0fde40d7206
            if(!entity().progress.enabled)return;
            if(read)background[0]=epok_timeline_adapter_detail::color(entity().progress.background[0]);else entity().progress.background[0]=epok_timeline_adapter_detail::color(background[0]);
            if(read)background[1]=epok_timeline_adapter_detail::color(entity().progress.background[1]);else entity().progress.background[1]=epok_timeline_adapter_detail::color(background[1]);
            if(read)background[2]=epok_timeline_adapter_detail::color(entity().progress.background[2]);else entity().progress.background[2]=epok_timeline_adapter_detail::color(background[2]);
            return;
        default:TimelineRect::timeline_sync(property,read);return;
        }
    }
};
