#pragma once
// Fixture for tests/integration/verify_lua_modes.py.
//
// A reflected user C++ base that Lua classes derive from. Every value a Lua
// body observes leaves the guest through `epok_lua_probe`, because a Lua body
// in the epok-lua profile cannot touch raw memory: the probe writers below are
// ordinary BlueprintCallable functions.
#include "epok.hpp"

extern "C" {
extern volatile int32_t epok_lua_probe[96];
void epok_lua_probe_begin();
void epok_lua_probe_end();
}

class EPOK_CLASS(Blueprintable, Id="6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d10") EnemyBase : public epok::Actor3D {
public:
    EPOK_PROPERTY(EditAnywhere, Id="6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d11") epok::Fixed health = 100.0;
    EPOK_PROPERTY(EditAnywhere, Id="6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d12") int32_t hits = 0;
    EPOK_PROPERTY(EditAnywhere, Id="6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d13") int32_t slot = 0;

    // Overridable in Lua; the default body is the one `epok.super` must reach.
    EPOK_FUNCTION(BlueprintEvent, BlueprintCallable, Id="6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d14")
    virtual void damage(epok::Fixed amount);
    EPOK_FUNCTION(BlueprintEvent, Id="6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d15")
    virtual void on_alert() {}
    // Not overridable: the frontend must refuse a Lua override of this.
    EPOK_FUNCTION(BlueprintCallable, Id="6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d16")
    virtual void sealed() final {}

    EPOK_FUNCTION(BlueprintCallable, Id="6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d17")
    void probe(int32_t index, int32_t value);
    EPOK_FUNCTION(BlueprintCallable, Id="6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d18")
    void probe_u(int32_t index, uint32_t value);
    EPOK_FUNCTION(BlueprintCallable, Id="6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d19")
    void probe_f(int32_t index, epok::Fixed value);
    EPOK_FUNCTION(BlueprintCallable, Id="6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d1a")
    void probe_b(int32_t index, bool value);
    // Side effect that short-circuit evaluation must NOT trigger.
    EPOK_FUNCTION(BlueprintCallable, Id="6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d1b")
    bool bump();
    // Returns 1, then 2, then 3...: pins the evaluation order of two calls.
    EPOK_FUNCTION(BlueprintCallable, Id="6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d1c")
    int32_t next_id();
    EPOK_FUNCTION(BlueprintCallable, Id="6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d1d")
    void mark_begin();
    EPOK_FUNCTION(BlueprintCallable, Id="6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d1e")
    void mark_end();

    void begin_play() override;
};

// Drives the run from a component hook, which the level calls once per rendered
// frame even while `epok::time` is paused, so the pause window can end.
class EPOK_CLASS(Blueprintable, Id="6a3c1f20-77d1-4b9e-8f42-9c0e1a5b7d20") Director : public epok::ActorComponent {
public:
    void frame_update(uint32_t elapsed) override;
    void tick(epok::Fixed dt) override;
private:
    uint32_t m_frame = 0;
};
