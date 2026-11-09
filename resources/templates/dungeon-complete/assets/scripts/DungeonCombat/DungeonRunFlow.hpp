#pragma once
#include "DungeonRunFlowBase.hpp"

// The C++ project's editable policy. Blueprint/Lua implement this same contract
// independently; none of these transitions belong to an engine-native service.
class EPOK_CLASS(Blueprintable,Owners=World3D,Id="df3f7e3e-b99c-5530-9d86-a44f87e67c5f") DungeonRunFlow:public DungeonRunFlowBase {
    bool timing_valid()const{
        const epok::Fixed values[]={timing.music_lead,timing.fade_in,timing.control_delay,timing.death_hold,timing.fade_out,timing.restart_hold};
        for(auto value:values)if(value<0.0||value>60.0)return false;
        return reactor_entry_z>=-4096.0&&reactor_entry_z<=4096.0;
    }
    void refresh(){
        status.valid=timing_valid()&&status.phase<=7&&status.state<=3&&status.elapsed>=0.0&&status.elapsed<=61.0;
        status.playing=status.phase==4;status.alive=status.state<2;status.restart=false;
        status.stop_music=status.phase==7;status.fade=0;status.music_gain=1.0;
        if(status.phase==0||status.phase==1||status.phase==7)status.fade=255;
        if(status.phase==2||status.phase==6){
            const auto duration=status.phase==2?timing.fade_in:timing.fade_out;
            const auto time=status.elapsed>duration?duration:status.elapsed;
            const uint32_t progress=duration>0.0?uint32_t(int64_t(time.raw())*255/duration.raw()):255;
            status.fade=status.phase==2?255-progress:progress;
        }
        if(status.phase==7)status.music_gain=0.0;
        if(status.phase==6){
            const auto time=status.elapsed>timing.fade_out?timing.fade_out:status.elapsed;
            status.music_gain=timing.fade_out>0.0?epok::Fixed(1.0)-time/timing.fade_out:epok::Fixed(0.0);
        }
    }
public:
    void start()override{status={};refresh();}
    void die()override{if(status.state>=2)return;status.state=3;status.phase=5;status.elapsed=0.0;refresh();}
    void win()override{if(status.state!=1)return;status.state=2;refresh();}
    bool try_enter_boss(bool has_key,epok::Fixed player_z)override{
        if(status.state!=0||!has_key||player_z<=reactor_entry_z)return false;
        status.state=1;refresh();return true;
    }
    bool restart_requested(bool pressed)const override{return pressed&&status.playing&&status.state>=2;}
    void advance(epok::Fixed dt,bool music_playing)override{
        if(dt<=0.0||dt>1.0)return;
        refresh();if(!status.valid)return;
        if(status.phase==0){if(music_playing){status.phase=1;status.elapsed=0.0;}refresh();return;}
        if(status.phase==4)return;
        epok::Fixed duration=timing.music_lead;
        if(status.phase==2)duration=timing.fade_in;
        if(status.phase==3)duration=timing.control_delay;
        if(status.phase==5)duration=timing.death_hold;
        if(status.phase==6)duration=timing.fade_out;
        if(status.phase==7)duration=timing.restart_hold;
        status.elapsed+=dt;
        if(status.elapsed<duration){refresh();return;}
        status.elapsed-=duration;
        if(status.phase==7){refresh();status.restart=true;return;}
        ++status.phase;if(status.phase==4)status.elapsed=0.0;
        refresh();
    }
};
