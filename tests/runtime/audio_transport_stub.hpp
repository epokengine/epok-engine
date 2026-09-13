#pragma once
#include <cstdint>
#include <cstddef>
namespace epok {
struct Fixed { int raw() const { return 4096; } };
struct AudioSource {
    bool enabled=true; int clip=-1; uint8_t priority=128; Fixed volume, pitch;
    void play(); void stop(); bool is_playing() const;
};
inline uint32_t transition_audio_gain(){return 4096;}
}
// Deterministic SPU/DMA transport: assignments complete immediately.
struct DmaControl {
    uint32_t value=0;
    void operator=(uint32_t v){value=v&~0x01000000;}
    operator uint32_t() const{return value;}
};
struct Dma {uint32_t MADR=0,BCR=0;DmaControl CHCR;};
inline Dma DMA_CTRL[1];
inline constexpr int DMA_SPU=0;
inline uint32_t DPCR=0,SBUS_DEV4_CTRL=0;
inline uint16_t SPU_CTRL=0,SPU_KEY_OFF_LOW=0,SPU_KEY_OFF_HIGH=0,
 SPU_VOL_MAIN_LEFT=0,SPU_VOL_MAIN_RIGHT=0,SPU_VOL_CD_LEFT=0,SPU_VOL_CD_RIGHT=0,
 SPU_VOL_EXT_LEFT=0,SPU_VOL_EXT_RIGHT=0,SPU_REVERB_LEFT=0,SPU_REVERB_RIGHT=0,
 SPU_REVERB_EN_LOW=0,SPU_REVERB_EN_HIGH=0,SPU_PITCH_MOD_LOW=0,SPU_PITCH_MOD_HIGH=0,
 SPU_NOISE_EN_LOW=0,SPU_NOISE_EN_HIGH=0,SPU_RAM_DTC=0,SPU_RAM_DTA=0,
 SPU_KEY_ON_LOW=0,SPU_KEY_ON_HIGH=0;
struct Voice {
    uint16_t volumeLeft=0,volumeRight=0,sampleStartAddr=0,sampleRepeatAddr=0,
    adsrLo=0,adsrHi=0,sampleRate=0;
};
inline Voice SPU_VOICES[24];
inline uint16_t audio_test_registers[512]{};
inline uint16_t& HW_U16(uintptr_t address){
    switch(address){
        case 0x1f801daa:case 0x1f801dae:return SPU_CTRL; // deterministic transfer-mode acknowledgement
        case 0x1f801d84:return SPU_REVERB_LEFT;
        case 0x1f801d86:return SPU_REVERB_RIGHT;
        case 0x1f801d98:return SPU_REVERB_EN_LOW;
        case 0x1f801d9a:return SPU_REVERB_EN_HIGH;
        case 0x1f801da6:return SPU_RAM_DTA;
        case 0x1f801dac:return SPU_RAM_DTC;
        default:return audio_test_registers[(address >> 1) & 511];
    }
}
