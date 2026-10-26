#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Owners=World3D,Id="582e4f6b-711a-487b-a3f4-831ba03c757e") SceneProbe:public epok::ActorComponent {
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("582e4f6b-711a-487b-a3f4-831ba03c757e");
    uint64_t class_id() const override{return static_class_id;}
    void begin_play() override;
    void tick(epok::Fixed) override;
    void end_play(epok::EndPlayReason) override;
};
