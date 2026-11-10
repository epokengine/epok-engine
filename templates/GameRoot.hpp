#pragma once
#include "epok.hpp"

// Add project rules here or derive another Actor class from this entry point.
class EPOK_CLASS(Blueprintable, Placeable, Spawnable, Domain=World3D, Id="509247ae-8730-52eb-98eb-e0a2436e6827") GameRoot : public epok::Actor3D {
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("509247ae-8730-52eb-98eb-e0a2436e6827");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
};
