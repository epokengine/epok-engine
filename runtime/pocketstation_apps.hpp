#pragma once
// The project's PocketStation apps, embedded in the game by the editor
// (the generated pocket-apps.hh). Include this from C++ behaviours:
//
//   if (const auto* app = epok::find_pocket_app("BEEPOKP12345STAR"))
//       epok::pocketstation.install_and_launch(*app);
#include "pocketstation.hpp"
#if __has_include("pocket-apps.hh")
#include "pocket-apps.hh"
#else
// Outside a game build (reflection, standalone checks) there are no apps.
namespace epok {
inline constexpr size_t pocket_app_count = 0;
inline const PocketApp* pocket_app_at(size_t) { return nullptr; }
namespace bp {
inline int pocket_app_index(uint64_t) { return -1; }
}  // namespace bp
}  // namespace epok
#endif

namespace epok {
// The embedded program with this memory card file name, or nullptr.
inline const PocketApp* find_pocket_app(const char* filename) {
  for (size_t i = 0; i < pocket_app_count; ++i) {
    const PocketApp* app = pocket_app_at(i);
    const char* a = app->filename;
    const char* b = filename;
    while (*a && *a == *b) ++a, ++b;
    if (*a == *b) return app;
  }
  return nullptr;
}
}  // namespace epok
