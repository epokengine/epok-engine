#pragma once
#include "epok.hpp"

// Scene-owned data only. These editable, derivable bindings contain no rules,
// callbacks, name lookup, UUID arithmetic or hidden provider implementations.

class EPOK_CLASS(Blueprintable,Owners=World3D,Id="c838c844-a361-512a-96e5-242bd7be91c4") DungeonAudioBindings:public epok::ActorComponent {
public:
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::AudioComponent",Id="60ae0d35-138c-5e17-b7ee-d6dacb955ad4") epok::ObjectId player_shot_source{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::AudioComponent",Id="f3561729-7bc9-542b-b2ca-2a5e0e963f76") epok::ObjectId world_hit_source{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::AudioComponent",Id="662d7f98-9f98-5630-8131-b513f0525f02") epok::ObjectId robot_hit_source{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::AudioComponent",Id="803f027a-a0fe-5b02-959f-1a04ba15510d") epok::ObjectId player_damage_source{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::AudioComponent",Id="22b755bd-b5b7-5627-8233-a49ad6dc9126") epok::ObjectId enemy_shot_source{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::AudioComponent",Id="32340795-2a7c-55cb-8a5d-7519a0a23638") epok::ObjectId robot_destroyed_source{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::AudioComponent",Id="ac4d1363-80a4-51d5-8fba-c4eacf0daeeb") epok::ObjectId ui_source{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::AudioComponent",Id="262c369f-11ce-59ec-b003-f13c6bdd0a56") epok::ObjectId music_source{};
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("c838c844-a361-512a-96e5-242bd7be91c4");
    uint64_t class_id()const override{return static_class_id;}
};

class EPOK_CLASS(Blueprintable,Owners=World3D,Id="3046197f-9623-56fa-8ee2-449e818130e0") DungeonEncounterBindings:public epok::ActorComponent {
public:
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="4193c96c-84a8-52d0-99a6-205b7f4dd5f4") epok::ObjectId guard0{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="fc4677e5-91e6-5104-8398-8eab7661f36b") epok::ObjectId guard1{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="2f630062-3b69-5249-8b69-0805e69db36e") epok::ObjectId guard2{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="baf507f4-6122-5613-9084-3b3f3745c0ff") epok::ObjectId boss{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="b25c1b61-f6ad-5cae-b857-a36aa87a42b2") epok::ObjectId key{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="3bd238c1-63ef-5232-a514-0c1c796ca9ec") epok::ObjectId pack0{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="6b0da5c8-1680-5c7b-ba88-12d7dd8569ba") epok::ObjectId pack1{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="61fb044e-44c7-5069-9e8e-f5acf6f2fc49") epok::ObjectId pack2{};
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("3046197f-9623-56fa-8ee2-449e818130e0");
    uint64_t class_id()const override{return static_class_id;}
};

class EPOK_CLASS(Blueprintable,Owners=World3D,Id="f622e994-4e5b-51dc-86c3-40bd52bba3f3") DungeonWorldBindings:public epok::ActorComponent {
public:
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="46599450-753a-58e1-b284-e20eeba427c0") epok::ObjectId door0_left{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="aef0e4c6-843b-58da-815e-a2fc44911bd8") epok::ObjectId door0_right{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="0509f79b-8f09-597b-b180-09b95501cac3") epok::ObjectId door1_left{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="e15adca4-87eb-5659-a140-c231fc7c414b") epok::ObjectId door1_right{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="624aa752-80b6-5c2c-89f3-d2e526dca883") epok::ObjectId door2_left{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="667b9d31-aea2-5794-9aaf-a576e439f0d5") epok::ObjectId door2_right{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="03526205-2d6a-5dc3-9aa7-38c01fbf6786") epok::ObjectId door3_left{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="87f191c6-ad54-5714-be35-aa88c661ffa7") epok::ObjectId door3_right{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="4cdbe3ff-c02b-5c69-8050-081bf1665163") epok::ObjectId room0{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="1c56a22c-9e2e-5cee-a132-602b4551058c") epok::ObjectId room1{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="4d6f348b-6295-577c-ab98-26b9540c7761") epok::ObjectId room2{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="9eedb91e-9864-5771-b0f9-497b5ce0b3e9") epok::ObjectId room3{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="625e9202-3d00-57fe-b52e-a8ce50209a09") epok::ObjectId room4{};
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("f622e994-4e5b-51dc-86c3-40bd52bba3f3");
    uint64_t class_id()const override{return static_class_id;}
};

class EPOK_CLASS(Blueprintable,Owners=World3D,Id="42a3ede0-d683-5c4a-b4dd-1a932654f76b") DungeonEffectBindings:public epok::ActorComponent {
public:
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="5cf39b8b-87e7-5d35-a232-5bce7f78772d") epok::ObjectId effect0{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="a86b0f44-61a9-57c9-b02c-cfe2ba19332f") epok::ObjectId effect1{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="139d7cff-d90e-5466-8c0e-b24121851b33") epok::ObjectId effect2{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="524a4854-0ba8-5a50-9e39-11b57dd53306") epok::ObjectId effect3{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="a40608cd-cbba-5538-aa90-7e444d58c369") epok::ObjectId effect4{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="c319133b-fde3-5905-beb5-fa930cd0db85") epok::ObjectId effect5{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="cdc94f35-c8bb-5e0b-808d-bb9ad2f12fbc") epok::ObjectId effect6{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="94fcf2b1-da5d-5791-9dbe-72470982d100") epok::ObjectId effect7{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="3795598c-9994-5121-aabe-2b3a28fb6b71") epok::ObjectId effect8{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="23349f90-3528-51cf-9151-70e7dab43674") epok::ObjectId effect9{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="6c06cb30-96d5-5d20-8b29-d118bd4ee1ff") epok::ObjectId effect10{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="df7a9012-75ad-583b-b5fd-45d00520fe2a") epok::ObjectId effect11{};
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("42a3ede0-d683-5c4a-b4dd-1a932654f76b");
    uint64_t class_id()const override{return static_class_id;}
};
