#pragma once
// The Blueprint/Lua operations that install and start the PocketStation
// programs embedded in the game (pocketstation_apps.hpp).
// Linked once by main.cpp.
#include "pocketstation_apps.hpp"

// Definitions of the blueprint_api.hpp declarations.
namespace epok::bp::api {
bool pocketstation_install(uint64_t app, uint32_t port) {
  const int index = app ? bp::pocket_app_index(app) : -1;
  if (index < 0) return false;
  return pocketstation.install(*pocket_app_at(size_t(index)), nullptr, port) == PocketError::OK;
}
bool pocketstation_start(uint64_t app, uint32_t port) {
  const int index = app ? bp::pocket_app_index(app) : -1;
  if (index < 0) return false;
  return pocketstation.install_and_launch(*pocket_app_at(size_t(index)), 0, port) == PocketError::OK;
}
}  // namespace epok::bp::api
