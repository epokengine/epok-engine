#pragma once
// PocketStation link for PlayStation games.
//
// A PocketStation in a memory card slot answers the ordinary memory card
// commands plus its own: probe, status, start a program, call one of the
// running program's link functions. Every request here is one short blocking
// transaction on the memory card bus (a few milliseconds), run on the main
// thread like PsyQo's blocking card accesses; do not call it from interrupts.
//
// Pocket Lua programs answer four link functions: 80h receives a message
// (link.received / link.byte), 81h returns the program's message
// (link.put / link.send), 82h reads its 16 shared values and 83h writes one
// (link.value / link.set_value). `send`, `receive`, `values` and `set_value`
// below speak exactly that protocol.
//
// In Epok Play with the editor's PocketStation attached, requests travel
// through `epok_pocket_mailbox` to the emulated PocketStation instead of SIO0.
#include <stddef.h>
#include <stdint.h>
#if defined(__mips__)
// The legacy common/hardware/hwregs.h macros DPCR and DICR would rename the
// PsyQo registers of the same name when that header was included first.
#pragma push_macro("DPCR")
#pragma push_macro("DICR")
#undef DPCR
#undef DICR
#include "psyqo/hardware/cpu.hh"
#include "psyqo/hardware/sio.hh"
#pragma pop_macro("DICR")
#pragma pop_macro("DPCR")
#endif

namespace epok {

enum class PocketError : uint8_t {
    OK = 0,
    NoCard,            // nothing answers in the slot
    NotPocketStation,  // a memory card that is not a PocketStation
    Timeout,           // the device stopped answering mid-request
    Refused,           // the device rejected the request (e.g. no such link function)
    BadReply,          // an answer did not follow the protocol
    NotFormatted,
    NotInstalled,      // the program is not on the card
    CardFull,
    Running,           // the program to replace is running on the PocketStation
    BadPort,
    BadArgument,
};

inline const char* pocket_error_message(PocketError error) {
    switch (error) {
        case PocketError::OK: return "OK";
        case PocketError::NoCard: return "No card in the slot";
        case PocketError::NotPocketStation: return "The card is not a PocketStation";
        case PocketError::Timeout: return "The PocketStation stopped answering";
        case PocketError::Refused: return "The PocketStation refused the request";
        case PocketError::BadReply: return "Unexpected answer from the PocketStation";
        case PocketError::NotFormatted: return "The card is not formatted";
        case PocketError::NotInstalled: return "The program is not on the card";
        case PocketError::CardFull: return "Not enough free blocks on the card";
        case PocketError::Running: return "The program is running on the PocketStation";
        case PocketError::BadPort: return "Invalid slot";
        case PocketError::BadArgument: return "Invalid argument";
    }
    return "Unknown error";
}

// Command 5Ah: what the PocketStation is doing and its clock.
struct PocketStatus {
    uint16_t running_block = 0;  // first block of the running program; 0 = start-up screen
    uint8_t flags = 0;
    uint32_t serial = 0;
    uint16_t year = 0;
    uint8_t month = 0, day = 0, hour = 0, minute = 0, second = 0;
    uint8_t weekday = 0;  // 1 = Sunday
};

// A PocketStation program embedded in the game, ready to install on a card.
struct PocketApp {
    const char* filename;  // memory card name, up to 20 characters, 7th character 'P'
    const uint8_t* image;  // whole 8 KiB blocks, as built by Epok
    uint32_t size;
};

// Exchange area with the editor's emulated PocketStation (Epok Play only).
struct PocketMailbox {
    static constexpr uint32_t magic_value = 0x504B5045;  // "EPKP"
    static constexpr unsigned max_frames = 16;
    static constexpr unsigned capacity = 2304;
    volatile uint32_t magic;
    volatile uint32_t request;
    volatile uint32_t answered;
    volatile uint32_t port;
    volatile uint32_t frames;
    volatile uint16_t lengths[max_frames];
    volatile uint16_t received[max_frames];
    volatile uint8_t send[capacity];
    volatile uint8_t reply[capacity];
};

}  // namespace epok

extern "C" {
// The editor finds this symbol in the link map and tells the emulator bridge.
// Games that never talk to a PocketStation link without it.
inline __attribute__((aligned(4))) epok::PocketMailbox epok_pocket_mailbox{};
}

namespace epok {

// One memory card transaction: `length` bytes out, replies in `reply`.
// `length_at` >= 0 names the reply byte that announces how many data bytes
// follow (commands 5Bh): the transaction then ends after that many plus one.
struct PocketFrame {
    const uint8_t* send = nullptr;
    uint8_t* reply = nullptr;
    uint16_t length = 0;
    int16_t length_at = -1;
    uint16_t received = 0;
};

namespace pocket_detail {
inline void delay(unsigned n) {
    for (unsigned i = 0; i < n; ++i) __asm__ volatile("");
}

// SIO0 transaction, polled like PsyQo's blocking memory card path. Builds
// for the PC (native Play, editor previews) have no memory card bus.
inline uint16_t sio_exchange(unsigned port, const uint8_t* send, uint8_t* reply, uint16_t length, int16_t length_at) {
#if !defined(__mips__)
    (void)port, (void)send, (void)reply, (void)length, (void)length_at;
    return 0;
#else
    using namespace psyqo::Hardware;
    const bool masked = CPU::IMask.isSet(CPU::IRQ::Controller);
    CPU::IMask.clear(CPU::IRQ::Controller);
    CPU::flushWriteQueue();
    SIO::Ctrl = 0;
    delay(2000);  // deselected recovery time between transactions
    SIO::Ctrl = (port ? SIO::Control::CTRL_PORTSEL : 0) | SIO::Control::CTRL_DTR;
    SIO::Baud = 0x88;  // 250 kHz
    while (SIO::Stat & SIO::Status::STAT_RXRDY) SIO::Data.throwAway();
    SIO::Ctrl |= (SIO::Control::CTRL_TXEN | SIO::Control::CTRL_ACKIRQEN);
    delay(100);
    uint16_t n = 0;
    uint16_t end = length;
    while (n < end) {
        SIO::Ctrl |= SIO::Control::CTRL_ERRRES;
        CPU::IReg.clear(CPU::IRQ::Controller);
        SIO::Data = send[n];
        while (!(SIO::Stat & SIO::Status::STAT_RXRDY));
        reply[n] = SIO::Data;
        if (int(n) == length_at) {
            const uint16_t total = uint16_t(n + 2 + reply[n]);
            if (total < end) end = total;
        }
        ++n;
        if (n == end) break;  // the last byte is never acknowledged
        // A command that runs program code on the PocketStation answers late.
        const uint32_t limit = n == 1 ? 0x4000u : 0x80000u;
        uint32_t spins = 0;
        while (!CPU::IReg.isSet(CPU::IRQ::Controller) && ++spins < limit);
        if (spins >= limit) break;
        uint32_t high = 0;
        while ((SIO::Stat & SIO::Status::STAT_ACK) && ++high < 0x4000);
    }
    SIO::Ctrl = 0;
    if (masked) CPU::IMask.set(CPU::IRQ::Controller);
    CPU::flushWriteQueue();
    return n;
#endif
}

inline bool mailbox_attached() { return epok_pocket_mailbox.magic == PocketMailbox::magic_value; }

// Hands frames to the emulator bridge and waits for its answer.
inline bool mailbox_exchange(unsigned port, PocketFrame* frames, unsigned count) {
    auto& box = epok_pocket_mailbox;
    unsigned total = 0;
    for (unsigned i = 0; i < count; ++i) total += frames[i].length;
    if (count > PocketMailbox::max_frames || total > PocketMailbox::capacity) return false;
    unsigned at = 0;
    for (unsigned i = 0; i < count; ++i) {
        box.lengths[i] = frames[i].length;
        for (unsigned b = 0; b < frames[i].length; ++b) box.send[at + b] = frames[i].send[b];
        at += frames[i].length;
    }
    box.port = port;
    box.frames = count;
    const uint32_t request = box.request + 1;
    box.request = request;
    // The bridge answers once per emulated frame; give up after a few seconds.
    for (uint32_t spins = 0; box.answered != request; ++spins) {
        if (spins > 40000000u) return false;
    }
    at = 0;
    for (unsigned i = 0; i < count; ++i) {
        uint16_t received = box.received[i];
        if (received > frames[i].length) received = frames[i].length;
        for (unsigned b = 0; b < received; ++b) frames[i].reply[b] = box.reply[at + b];
        frames[i].received = received;
        at += frames[i].length;
    }
    return true;
}
}  // namespace pocket_detail

class PocketStationService {
  public:
    static constexpr uint8_t message_capacity = 64;
    static constexpr unsigned value_count = 16;

    // Runs memory card transactions on `port` (0 = first slot).
    void transfer(unsigned port, PocketFrame* frames, unsigned count) {
        if (pocket_detail::mailbox_attached()) {
            if (pocket_detail::mailbox_exchange(port, frames, count)) return;
            for (unsigned i = 0; i < count; ++i) frames[i].received = 0;
            return;
        }
        for (unsigned i = 0; i < count; ++i) {
            frames[i].received = pocket_detail::sio_exchange(port, frames[i].send, frames[i].reply, frames[i].length, frames[i].length_at);
        }
    }

    // True when a PocketStation (not a plain memory card) is in the slot.
    PocketError probe(unsigned port = 0) {
        if (port > 1) return PocketError::BadPort;
        uint8_t send[5] = {0x81, 0x58, 0, 0, 0}, reply[5] = {};
        if (!one(port, send, reply, 5)) return last_;
        if (received_ < 2) return PocketError::NoCard;
        if (received_ < 5 || reply[2] != 0x02) return PocketError::NotPocketStation;
        return PocketError::OK;
    }
    bool connected(unsigned port = 0) { return probe(port) == PocketError::OK; }

    PocketError status(PocketStatus& out, unsigned port = 0) {
        uint8_t send[21] = {0x81, 0x5A}, reply[21] = {};
        if (!one(port, send, reply, 21)) return last_;
        if (received_ < 2) return PocketError::NoCard;
        if (received_ < 21 || reply[2] != 0x12) return PocketError::NotPocketStation;
        const uint8_t* d = reply + 3;
        out.running_block = uint16_t((d[0] << 8) | d[1]);
        out.flags = uint8_t(d[2] | (d[3] << 1) | (d[5] << 2) | (d[4] << 3));
        out.serial = uint32_t(d[6]) | (uint32_t(d[7]) << 8) | (uint32_t(d[8]) << 16) | (uint32_t(d[9]) << 24);
        out.day = bcd(d[10]);
        out.month = bcd(d[11]);
        out.year = uint16_t(bcd(d[13]) * 100 + bcd(d[12]));
        out.second = bcd(d[14]);
        out.minute = bcd(d[15]);
        out.hour = bcd(d[16]);
        out.weekday = d[17];
        return PocketError::OK;
    }

    // Starts the program whose first block is `block` (0 = start-up screen).
    // The PocketStation switches when the running program allows it.
    PocketError launch(uint8_t block, uint32_t parameter = 0, unsigned port = 0) {
        if (block > 15) return PocketError::BadArgument;
        uint8_t send[9] = {0x81, 0x59, 0, 0, block, uint8_t(parameter), uint8_t(parameter >> 8), uint8_t(parameter >> 16), uint8_t(parameter >> 24)};
        uint8_t reply[9] = {};
        if (!one(port, send, reply, 9)) return last_;
        if (received_ < 2) return PocketError::NoCard;
        return received_ == 9 && reply[2] == 0x06 ? PocketError::OK : PocketError::NotPocketStation;
    }

    // Calls link function `function` of the running program. With
    // `to_playstation` the program's data is read into `data` (up to
    // `capacity` bytes, `*size` receives the count); otherwise `size` bytes of
    // `data` are sent. `parameters` are sent first.
    PocketError call(uint8_t function, bool to_playstation, const uint8_t* parameters, uint8_t parameter_count, uint8_t* data, uint8_t capacity, uint8_t* size, unsigned port = 0) {
        if (parameter_count > 128 || capacity > 128) return PocketError::BadArgument;
        uint8_t send[4 + 128 + 1 + 128 + 1] = {}, reply[sizeof(send)] = {};
        send[0] = 0x81;
        send[1] = to_playstation ? 0x5B : 0x5C;
        send[2] = function;
        for (unsigned i = 0; i < parameter_count; ++i) send[4 + i] = parameters[i];
        const unsigned data_at = 5 + parameter_count;
        const uint8_t outgoing = to_playstation ? capacity : (size ? *size : 0);
        if (!to_playstation)
            for (unsigned i = 0; i < outgoing; ++i) send[data_at + i] = data[i];
        const uint16_t length = uint16_t(data_at + outgoing + 1);
        PocketFrame frame{send, reply, length, int16_t(to_playstation ? data_at - 1 : -1)};
        transfer(port, &frame, 1);
        if (frame.received < 2) return PocketError::NoCard;
        if (frame.received < 4 || reply[2] != 0xFF) return PocketError::Refused;
        if (reply[3] != parameter_count) return PocketError::BadReply;
        if (frame.received <= data_at - 1) return PocketError::Timeout;
        const uint8_t count = reply[data_at - 1];
        if (to_playstation) {
            const uint8_t copied = count < capacity ? count : capacity;
            for (unsigned i = 0; i < copied; ++i) data[i] = reply[data_at + i];
            if (size) *size = copied;
            if (frame.received < data_at + count + 1u) return PocketError::Timeout;
        } else {
            if (count != outgoing) return PocketError::BadReply;
            if (frame.received < length) return PocketError::Timeout;
        }
        return PocketError::OK;
    }

    // Pocket Lua link: a message for `link.received()` / `link.byte(i)`.
    PocketError send(const void* message, uint8_t size, unsigned port = 0) {
        if (size == 0 || size > message_capacity) return PocketError::BadArgument;
        uint8_t length = size;
        return call(0x80, false, &length, 1, const_cast<uint8_t*>(static_cast<const uint8_t*>(message)), size, &length, port);
    }
    // Pocket Lua link: the message published with `link.send(n)`; size 0 when none.
    PocketError receive(void* message, uint8_t capacity, uint8_t* size, unsigned port = 0) {
        return call(0x81, true, nullptr, 0, static_cast<uint8_t*>(message), capacity, size, port);
    }
    // Pocket Lua link: the 16 values set with `link.set_value(i, v)`.
    PocketError values(int32_t out[value_count], unsigned port = 0) {
        uint8_t bytes[value_count * 4] = {};
        uint8_t size = 0;
        PocketError e = call(0x82, true, nullptr, 0, bytes, sizeof(bytes), &size, port);
        if (e != PocketError::OK) return e;
        for (unsigned i = 0; i < value_count; ++i) out[i] = int32_t(get32(bytes + i * 4));
        return PocketError::OK;
    }
    // Pocket Lua link: sets value `index` (1-16), read with `link.value(index)`.
    PocketError set_value(uint8_t index, int32_t value, unsigned port = 0) {
        if (index < 1 || index > value_count) return PocketError::BadArgument;
        uint8_t bytes[4];
        put32(bytes, uint32_t(value));
        uint8_t size = 4;
        return call(0x83, false, &index, 1, bytes, 4, &size, port);
    }

    // Kernel memory access (RAM, I/O, flash), up to 128 bytes.
    PocketError read_memory(uint32_t address, void* out, uint8_t size, unsigned port = 0) {
        uint8_t parameters[5];
        put32(parameters, address);
        parameters[4] = size;
        uint8_t got = 0;
        PocketError e = call(0x01, true, parameters, 5, static_cast<uint8_t*>(out), size, &got, port);
        return e == PocketError::OK && got != size ? PocketError::BadReply : e;
    }
    PocketError write_memory(uint32_t address, const void* data, uint8_t size, unsigned port = 0) {
        uint8_t parameters[5];
        put32(parameters, address);
        parameters[4] = size;
        return call(0x01, false, parameters, 5, const_cast<uint8_t*>(static_cast<const uint8_t*>(data)), size, &size, port);
    }
    // The red LED (through the PocketStation's I/O port).
    PocketError set_led(bool on, unsigned port = 0) {
        const uint8_t bit = 0x02;
        return write_memory(on ? 0x0D800008u : 0x0D800004u, &bit, 1, port);
    }
    // Sets the clock (year 1900-2099).
    PocketError set_time(uint16_t year, uint8_t month, uint8_t day, uint8_t hour, uint8_t minute, uint8_t second, uint8_t weekday, unsigned port = 0) {
        uint8_t data[8] = {to_bcd(day), to_bcd(month), to_bcd(uint8_t(year % 100)), to_bcd(uint8_t(year / 100)), to_bcd(second), to_bcd(minute), to_bcd(hour), weekday};
        uint8_t size = 8;
        return call(0x00, false, nullptr, 0, data, 8, &size, port);
    }

    // Reads the 16 directory frames (block 0) in one batch.
    PocketError read_directory(uint8_t (*out)[128], unsigned port = 0) {
        if (port > 1) return PocketError::BadPort;
        static uint8_t send[16][140], reply[16][140];
        PocketFrame frames[16];
        for (unsigned b = 0; b < 16; ++b) {
            for (unsigned i = 0; i < 140; ++i) send[b][i] = 0;
            send[b][0] = 0x81;
            send[b][1] = 0x52;
            send[b][5] = uint8_t(b);
            frames[b] = PocketFrame{send[b], reply[b], 140, -1};
        }
        transfer(port, frames, 16);
        for (unsigned b = 0; b < 16; ++b) {
            if (frames[b].received < 2) return PocketError::NoCard;
            if (frames[b].received < 140 || reply[b][139] != 0x47) return PocketError::BadReply;
            for (unsigned i = 0; i < 128; ++i) out[b][i] = reply[b][10 + i];
        }
        if (out[0][0] != 'M' || out[0][1] != 'C') return PocketError::NotFormatted;
        return PocketError::OK;
    }

    // Finds a file by name; `*block` receives its first block.
    PocketError find(const char* filename, uint8_t* block, unsigned port = 0) {
        static uint8_t directory[16][128];
        PocketError e = read_directory(directory, port);
        if (e != PocketError::OK) return e;
        for (uint8_t b = 1; b < 16; ++b) {
            if (directory[b][0] == 0x51 && same_name(directory[b] + 10, filename)) {
                if (block) *block = b;
                return PocketError::OK;
            }
        }
        return PocketError::NotInstalled;
    }

    // Copies an embedded program to the card (replacing an older copy) and
    // returns its first block. Takes about a second for a one-block program.
    PocketError install(const PocketApp& app, uint8_t* first_block = nullptr, unsigned port = 0) {
        if (!app.filename || !app.image || app.size == 0 || app.size % 8192) return PocketError::BadArgument;
        const unsigned needed = app.size / 8192;
        static uint8_t directory[16][128], original[16][128];
        PocketError status_error = read_directory(directory, port);
        if (status_error != PocketError::OK) return status_error;
        for (unsigned b = 0; b < 16; ++b)
            for (unsigned c = 0; c < 128; ++c) original[b][c] = directory[b][c];
        PocketStatus state;
        const bool pocketstation = status(state, port) == PocketError::OK;
        // Free the blocks of an older copy (unless it is running).
        for (uint8_t b = 1; b < 16; ++b) {
            if (directory[b][0] != 0x51 || !same_name(directory[b] + 10, app.filename)) continue;
            if (pocketstation && state.running_block == b) return PocketError::Running;
            uint8_t at = b;
            for (unsigned guard = 0; guard < 15 && at >= 1 && at < 16; ++guard) {
                uint16_t next = uint16_t(directory[at][8] | (directory[at][9] << 8));
                directory[at][0] = 0xA0;
                if (next == 0xFFFF) break;
                at = uint8_t(next + 1);
            }
        }
        uint8_t blocks[15];
        unsigned found = 0;
        for (uint8_t b = 1; b < 16 && found < needed; ++b)
            if ((directory[b][0] & 0xF0) == 0xA0) blocks[found++] = b;
        if (found < needed) return PocketError::CardFull;
        // Data first, directory last: an interrupted copy leaves no file.
        PocketFrame frames[PocketMailbox::max_frames];
        static uint8_t send[PocketMailbox::max_frames][138], reply[PocketMailbox::max_frames][138];
        unsigned pending = 0;
        for (unsigned i = 0; i < needed; ++i) {
            for (unsigned s = 0; s < 64; ++s) {
                const uint16_t sector = uint16_t(blocks[i] * 64 + s);
                write_frame(send[pending], sector, app.image + i * 8192 + s * 128);
                frames[pending] = PocketFrame{send[pending], reply[pending], 138, -1};
                if (++pending == PocketMailbox::max_frames) {
                    PocketError e = flush_writes(port, frames, pending);
                    if (e != PocketError::OK) return e;
                    pending = 0;
                }
            }
        }
        if (pending) {
            PocketError e = flush_writes(port, frames, pending);
            if (e != PocketError::OK) return e;
        }
        for (unsigned i = 0; i < needed; ++i) {
            uint8_t frame[128] = {};
            frame[0] = i == 0 ? 0x51 : (i + 1 == needed ? 0x53 : 0x52);
            if (i == 0) {
                put32(frame + 4, needed * 8192);
                for (unsigned c = 0; c < 20 && app.filename[c]; ++c) frame[10 + c] = uint8_t(app.filename[c]);
            }
            const uint16_t next = i + 1 < needed ? uint16_t(blocks[i + 1] - 1) : 0xFFFF;
            frame[8] = uint8_t(next);
            frame[9] = uint8_t(next >> 8);
            for (unsigned c = 0; c < 127; ++c) frame[127] ^= frame[c];
            for (unsigned c = 0; c < 128; ++c) directory[blocks[i]][c] = frame[c];
        }
        // Rewrite every directory frame that changed (new file and freed copies).
        pending = 0;
        for (uint8_t b = 1; b < 16; ++b) {
            uint8_t checksum = 0;
            for (unsigned c = 0; c < 127; ++c) checksum ^= directory[b][c];
            directory[b][127] = checksum;
            bool same = true;
            for (unsigned c = 0; c < 128 && same; ++c) same = original[b][c] == directory[b][c];
            if (same) continue;
            write_frame(send[pending], b, directory[b]);
            frames[pending] = PocketFrame{send[pending], reply[pending], 138, -1};
            ++pending;
        }
        if (pending) {
            PocketError e = flush_writes(port, frames, pending);
            if (e != PocketError::OK) return e;
        }
        if (first_block) *first_block = blocks[0];
        return PocketError::OK;
    }

    // Installs (if needed) and starts an embedded program.
    PocketError install_and_launch(const PocketApp& app, uint32_t parameter = 0, unsigned port = 0) {
        uint8_t block = 0;
        PocketError e = find(app.filename, &block, port);
        if (e == PocketError::NotInstalled) e = install(app, &block, port);
        if (e != PocketError::OK) return e;
        return launch(block, parameter, port);
    }

    PocketError read_sector(uint16_t sector, uint8_t* out, unsigned port = 0) {
        uint8_t send[140] = {0x81, 0x52, 0, 0, uint8_t(sector >> 8), uint8_t(sector)}, reply[140] = {};
        if (!one(port, send, reply, 140)) return last_;
        if (received_ < 2) return PocketError::NoCard;
        if (received_ < 140 || reply[139] != 0x47) return PocketError::BadReply;
        for (unsigned i = 0; i < 128; ++i) out[i] = reply[10 + i];
        return PocketError::OK;
    }
    PocketError write_sector(uint16_t sector, const uint8_t* data, unsigned port = 0) {
        uint8_t send[138], reply[138] = {};
        write_frame(send, sector, data);
        if (!one(port, send, reply, 138)) return last_;
        if (received_ < 2) return PocketError::NoCard;
        if (received_ < 138) return PocketError::Timeout;
        return reply[137] == 0x47 ? PocketError::OK : (reply[137] == 0xFD ? PocketError::Running : PocketError::Refused);
    }

  private:
    PocketError last_ = PocketError::OK;
    uint16_t received_ = 0;

    bool one(unsigned port, const uint8_t* send, uint8_t* reply, uint16_t length) {
        if (port > 1) {
            last_ = PocketError::BadPort;
            return false;
        }
        PocketFrame frame{send, reply, length, -1};
        transfer(port, &frame, 1);
        received_ = frame.received;
        return true;
    }
    PocketError flush_writes(unsigned port, PocketFrame* frames, unsigned count) {
        transfer(port, frames, count);
        for (unsigned i = 0; i < count; ++i) {
            if (frames[i].received < 2) return PocketError::NoCard;
            if (frames[i].received < 138) return PocketError::Timeout;
            if (frames[i].reply[137] != 0x47) return PocketError::Refused;
        }
        return PocketError::OK;
    }
    static void write_frame(uint8_t* send, uint16_t sector, const uint8_t* data) {
        send[0] = 0x81;
        send[1] = 0x57;
        send[2] = send[3] = 0;
        send[4] = uint8_t(sector >> 8);
        send[5] = uint8_t(sector);
        uint8_t checksum = uint8_t(send[4] ^ send[5]);
        for (unsigned i = 0; i < 128; ++i) {
            send[6 + i] = data[i];
            checksum ^= data[i];
        }
        send[134] = checksum;
        send[135] = send[136] = send[137] = 0;
    }
    static bool same_name(const uint8_t* stored, const char* name) {
        for (unsigned i = 0; i < 20; ++i) {
            if (uint8_t(name[i]) != stored[i]) return false;
            if (!name[i]) return true;
        }
        return name[20] == 0;
    }
    static uint8_t bcd(uint8_t v) { return uint8_t((v >> 4) * 10 + (v & 15)); }
    static uint8_t to_bcd(uint8_t v) { return uint8_t(((v / 10) << 4) | (v % 10)); }
    static uint32_t get32(const uint8_t* p) { return uint32_t(p[0]) | (uint32_t(p[1]) << 8) | (uint32_t(p[2]) << 16) | (uint32_t(p[3]) << 24); }
    static void put32(uint8_t* p, uint32_t v) {
        for (unsigned i = 0; i < 4; ++i) p[i] = uint8_t(v >> (i * 8));
    }
};

inline PocketStationService pocketstation;

}  // namespace epok
