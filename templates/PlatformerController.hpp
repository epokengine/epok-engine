#pragma once
#include "epok.hpp"

// Project input extension. Override calculate_command to add project rules;
// Character2D's motor and the sprite driver own simulation and presentation.
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Id="ca7559df-c335-44a1-ae8c-5268c5b68ace") PlatformerController : public epok::LocalPlayer2DInputComponent {
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("ca7559df-c335-44a1-ae8c-5268c5b68ace");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
};
