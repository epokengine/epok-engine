#pragma once
#include "epok.hpp"

// A reflected C++ Actor3D that Lua classes can extend. Nothing here is
// Lua-specific: eligibility is the ordinary authoring rule — `Blueprintable`,
// not `final`, native backend — so the same base also works for Blueprints.
class EPOK_CLASS(Blueprintable, Id="08ed74b3-66ae-4099-ba8c-09e9a07c10cf") EnemyBase
    : public epok::Actor3D {
public:
    static constexpr uint64_t static_class_id =
        epok::detail::compact_class_id("08ed74b3-66ae-4099-ba8c-09e9a07c10cf");
    uint64_t class_id() const override { return static_class_id; }

    // Inspector-editable Q12 storage. A Lua child reads and writes it as
    // `self.health`; the field itself stays in this native base.
    EPOK_PROPERTY(EditAnywhere, Id="f5f724d7-762b-4aec-a7cf-fd9571a898e2")
    epok::Fixed health = 100.0;

    // Reachable from a Lua body as `self:apply_damage(amount)`.
    EPOK_FUNCTION(BlueprintCallable, Id="137aa9bc-ec12-4b64-8229-0441f59d46de")
    void apply_damage(epok::Fixed amount) { health = health - amount; }

    // A virtual event a Lua child may override instead of calling.
    EPOK_FUNCTION(BlueprintEvent, Id="a6bb20f5-5f48-40b3-a031-d295cd1f93c3")
    virtual void defeated() {}
};
