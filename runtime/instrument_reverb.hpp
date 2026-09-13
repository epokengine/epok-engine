#pragma once
// Exclusive PSX SPU reverb resource. prepare/acquire/reap/teardown are
// main-thread operations because they may perform bounded DMA. send and
// clear_voice only write the EON mask and are safe for the audio service path.
//
// The production hardware adapter names registers by HW_U16 address so tests
// can instantiate Resource with a mock without depending on SDK SPU aliases.
#include "common/hardware/dma.h"
#include "common/hardware/hwregs.h"
#include <cstddef>
#include <cstdint>

namespace epok::instrument::reverb {

inline constexpr uint32_t spu_bytes = 512 * 1024;
inline constexpr uint32_t room_bytes = 0x26c0;
inline constexpr uint32_t room_begin = spu_bytes - room_bytes;
inline constexpr uint16_t no_owner = 0xffff;

enum class Preset : uint8_t { Dry, Room };
enum class Error : uint8_t {
    None,
    InvalidPreset,
    InvalidDepth,
    InvalidOwner,
    SampleBudgetConflict,
    NotPrepared,
    LeaseConflict,
    NotOwner,
    InvalidVoice,
    DmaTimeout,
};

// Sony Room example as documented by psx-spx for 1F801DC0h..1F801DFFh.
inline constexpr uint16_t room_registers[32] = {
    0x007d,0x005b,0x6d80,0x54b8,0xbed0,0x0000,0x0000,0xba80,
    0x5800,0x5300,0x04d6,0x0333,0x03f0,0x0227,0x0374,0x01ef,
    0x0334,0x01b5,0x0000,0x0000,0x0000,0x0000,0x0000,0x0000,
    0x0000,0x0000,0x01b4,0x0136,0x00b8,0x005c,0x8000,0x8000,
};

struct PsxHardware {
    static uint16_t read16(uintptr_t address) { return HW_U16(address); }
    static void write16(uintptr_t address, uint16_t value) { HW_U16(address) = value; }

    static bool zero_spu(uint32_t address, uint32_t bytes) {
        if ((address & 63) || !bytes || (bytes & 63) || address > spu_bytes || bytes > spu_bytes - address || bytes > room_bytes)
            return false;
        // A single bounded DMA avoids 155 transfer/address handshakes during
        // an owner handoff. This deliberately costs 9,920 bytes of main BSS.
        alignas(64) static uint32_t zero_buffer[room_bytes/4]{};
        constexpr uint32_t timeout = 10000000;
        uint32_t remaining = timeout;
        DPCR = DPCR | 0x000b0000;
        HW_U16(0x1f801dac) = 4;
        const uint16_t original = HW_U16(0x1f801daa);
        const uint16_t dma_write = uint16_t((original & ~0x0030u) | 0x0020u);
        HW_U16(0x1f801daa) = dma_write;
        while ((HW_U16(0x1f801dae) & 0x003fu) != (dma_write & 0x003fu) && --remaining) {}
        if (!remaining) { HW_U16(0x1f801daa) = uint16_t(original & ~0x0030u); return false; }
        {
            HW_U16(0x1f801da6) = uint16_t(address >> 3);
            DMA_CTRL[DMA_SPU].MADR = uint32_t(uintptr_t(zero_buffer));
            DMA_CTRL[DMA_SPU].BCR = ((bytes/64) << 16) | 16u;
            DMA_CTRL[DMA_SPU].CHCR = 0x01000201;
            while ((DMA_CTRL[DMA_SPU].CHCR & 0x01000000) && --remaining) {}
            // DMA completion only means the SPU FIFO accepted the last words.
            while ((HW_U16(0x1f801dae) & 0x0400u) && --remaining) {}
            if (!remaining) {
                DMA_CTRL[DMA_SPU].CHCR = 0;
                HW_U16(0x1f801daa) = uint16_t(original & ~0x0030u);
                return false;
            }
        }
        const uint16_t stopped = uint16_t(original & ~0x0030u);
        HW_U16(0x1f801daa) = stopped;
        while ((HW_U16(0x1f801dae) & 0x003fu) != (stopped & 0x003fu) && --remaining) {}
        return remaining != 0;
    }
};

template<class Hardware = PsxHardware>
class Resource {
public:
    Error prepare(uint32_t sample_upload_end, Preset preset, uint16_t depth_q15) {
        if (preset != Preset::Dry && preset != Preset::Room) return Error::InvalidPreset;
        if (depth_q15 > 32767 || (preset == Preset::Dry && depth_q15)) return Error::InvalidDepth;
        // Reject a colliding reservation before muting or changing a resource
        // that callers may continue to use after this recoverable error.
        if (preset == Preset::Room && sample_upload_end > room_begin) return Error::SampleBudgetConflict;
        if (owner_ != no_owner) return Error::LeaseConflict;
        disable();
        if (preset == Preset::Dry) {
            if (prepared_ && preset_ == Preset::Room && !zero_buffer()) {
                dirty_ = true;
                return Error::DmaTimeout;
            }
            clear_state(); prepared_ = true; return Error::None;
        }
        preset_ = preset; depth_q15_ = depth_q15; sample_upload_end_ = sample_upload_end;
        write_configuration();
        if (!zero_buffer()) { clear_state(); return Error::DmaTimeout; }
        dirty_ = false; prepared_ = true;
        return Error::None;
    }

    Error acquire(uint16_t owner, uint32_t generation) {
        if (owner == no_owner) return Error::InvalidOwner;
        if (!prepared_) return Error::NotPrepared;
        if (preset_ == Preset::Dry) return Error::None;
        if (owner_ != no_owner)
            return owner_ == owner && generation_ == generation ? Error::None : Error::LeaseConflict;
        disable();
        if (dirty_ && !zero_buffer()) return Error::DmaTimeout;
        write_configuration();
        owner_ = owner; generation_ = generation; dirty_ = false;
        Hardware::write16(0x1f801d84, depth_q15_);
        Hardware::write16(0x1f801d86, depth_q15_);
        Hardware::write16(0x1f801daa, uint16_t(Hardware::read16(0x1f801daa) | 0x0080u));
        return Error::None;
    }

    Error release(uint16_t owner, uint32_t generation) {
        if (owner_ == no_owner || owner_ != owner || generation_ != generation) return Error::NotOwner;
        disable(); owner_ = no_owner; generation_ = 0; dirty_ = true;
        return Error::None;
    }
    Error reap(uint16_t owner, uint32_t generation) { return release(owner, generation); }

    Error send(uint8_t physical_voice, bool enabled) {
        if (physical_voice >= 24) return Error::InvalidVoice;
        const uint32_t bit = uint32_t(1) << physical_voice;
        if(((send_mask_&bit)!=0)==enabled)return Error::None;
        if (enabled) {
            if (owner_ == no_owner || preset_ != Preset::Room) return Error::NotOwner;
            send_mask_ |= bit;
        } else send_mask_ &= ~bit;
        write_send_mask();
        return Error::None;
    }
    void clear_voice(uint8_t physical_voice) {
        if (physical_voice >= 24) return;
        if(!(send_mask_&(uint32_t(1)<<physical_voice)))return;
        send_mask_ &= ~(uint32_t(1) << physical_voice);
        write_send_mask();
    }

    Error teardown() {
        disable();
        const bool needs_zero = prepared_ && preset_ == Preset::Room;
        owner_ = no_owner; generation_ = 0;
        if (needs_zero && !zero_buffer()) { dirty_ = true; return Error::DmaTimeout; }
        clear_state(); return Error::None;
    }

    bool prepared() const { return prepared_; }
    bool active() const { return owner_ != no_owner && preset_ == Preset::Room; }
    Preset preset() const { return preset_; }
    uint16_t owner() const { return owner_; }
    uint32_t generation() const { return generation_; }
    uint32_t send_mask() const { return send_mask_; }
    uint32_t reserved_begin() const { return preset_ == Preset::Room ? room_begin : spu_bytes; }
    uint32_t reserved_bytes() const { return preset_ == Preset::Room ? room_bytes : 0; }

private:
    void write_send_mask() {
        Hardware::write16(0x1f801d98, uint16_t(send_mask_));
        Hardware::write16(0x1f801d9a, uint16_t(send_mask_ >> 16));
    }
    void disable() {
        send_mask_ = 0; write_send_mask();
        Hardware::write16(0x1f801d84, 0); Hardware::write16(0x1f801d86, 0);
        Hardware::write16(0x1f801daa, uint16_t(Hardware::read16(0x1f801daa) & ~0x0080u));
    }
    void write_configuration() {
        Hardware::write16(0x1f801da2, uint16_t(room_begin >> 3));
        Hardware::write16(0x1f801dac, 4);
        for (uint16_t i = 0; i < 32; ++i)
            Hardware::write16(0x1f801dc0 + uintptr_t(i) * 2, room_registers[i]);
    }
    bool zero_buffer() { return Hardware::zero_spu(room_begin, room_bytes); }
    void clear_state() {
        preset_ = Preset::Dry; depth_q15_ = 0; sample_upload_end_ = 0;
        owner_ = no_owner; generation_ = 0; send_mask_ = 0;
        dirty_ = false; prepared_ = false;
    }

    Preset preset_ = Preset::Dry;
    uint16_t depth_q15_ = 0;
    uint16_t owner_ = no_owner;
    uint32_t generation_ = 0;
    uint32_t sample_upload_end_ = 0;
    uint32_t send_mask_ = 0;
    bool dirty_ = false;
    bool prepared_ = false;
};

} // namespace epok::instrument::reverb
