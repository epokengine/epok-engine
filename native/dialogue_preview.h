#pragma once
#include "hud_preview.h"

// Borrowed, already validated/cooked authoring data. Preview callbacks only read
// explicit values or append trace records; project code is never linked here.
struct EpokDialoguePage {const char* text;uint32_t glyphs;};
struct EpokDialogueText {uint32_t first,count;};
struct EpokDialogueRole {uint32_t id;int32_t name,portrait;};
struct EpokDialogueBinding {uint32_t id,role,kind,value;};
struct EpokDialogueOption {uint32_t id;int32_t text,next,binding;};
struct EpokDialogueNode {
 uint32_t id,kind;int32_t speaker,text,next,otherwise,binding;
 uint32_t first_option,option_count,reveal_ms,auto_ms;
};
struct EpokDialogueGraph {
 const EpokDialogueRole* roles;uint32_t role_count;
 const EpokDialogueBinding* bindings;uint32_t binding_count;
 const EpokDialoguePage* pages;uint32_t page_count;
 const EpokDialogueText* texts;uint32_t text_count;
 const EpokDialogueOption* options;uint32_t option_count;
 const EpokDialogueNode* nodes;uint32_t node_count,entry;
};
struct EpokDialogueFrame {uint32_t buttons,microseconds;};
struct EpokDialogueRequest {
 EpokDialogueGraph graph;
 const EpokHudNode* nodes;uint32_t count,root;
 const uint32_t* components;
 const int32_t* logical_parents;
 int32_t controls[11];
 const EpokDialogueFrame* frames;uint32_t frame_count,reveal_ms,auto_ms;
 const int32_t* dimensions;uint32_t texture_count;
 int32_t width,height;
 const uint32_t* budget;
 const int32_t* fonts;uint32_t font_words;
};
struct EpokDialogueSnapshot {
 uint32_t phase,result,start_result,node,revealed,glyphs,page,pages,choices,choice_page,selected,trace_count;
};
struct EpokDialogueTrace {uint32_t kind,id,node;};
extern "C" uint32_t epok_dialogue_preview(const EpokDialogueRequest*,EpokHudCommand*,uint32_t,uint32_t*,EpokDialogueSnapshot*,EpokDialogueTrace*,uint32_t,char*,uint32_t);
