TARGET = epok
TYPE = ps-exe
include sources.mk
SRCS = main.cpp $(SCRIPT_SRCS)
CPPFLAGS += -I. -Iscripts
ifeq ($(EPOK_VALIDATE_GTE),1)
CPPFLAGS += -DEPOK_VALIDATE_GTE
endif
ifeq ($(EPOK_PROFILE_DETAIL),1)
CPPFLAGS += -DEPOK_PROFILE_DETAIL
endif
NUGGET_DIR ?= ../../third_party/nugget
include $(NUGGET_DIR)/psyqo/psyqo.mk

# Speed-optimize scene traversal and gameplay on MIPS, keeping the SDK compact.
# Overrides permit A/B profiling; the final executable still passes the RAM gate.
EPOK_RUNTIME_OPT ?= -O2
EPOK_SCRIPT_OPT ?= -O2
ifeq ($(BUILD),Release)
main.o main.dep: CPPFLAGS += $(EPOK_RUNTIME_OPT)
$(SCRIPT_SRCS:.cpp=.o) $(SCRIPT_SRCS:.cpp=.dep): CPPFLAGS += $(EPOK_SCRIPT_OPT)
endif
main.o: Makefile

# MIPS has no 64-bit divide instruction. Collision, clipping and measured time
# use GCC's integer helpers; append the archive after application/PsyQo objects.
LIBRARIES += $(shell $(CC) -print-libgcc-file-name)

# sources.mk sets EPOK_LUA_VM only for the two Lua VM execution modes; the
# archive it builds is the only interpreter the executable ever links.
ifeq ($(EPOK_LUA_VM),1)
include lua.mk
endif

# Relink when the set of scripts changes, including script removal.
epok.elf: sources.mk

include build-inputs.mk

# A plain standalone `make` uses a fresh, isolated SDK build. The editor supplies
# an already certified archive and invokes explicit targets instead.
ifeq ($(EPOK_CERTIFIED_SDK),)
.DEFAULT_GOAL := standalone
.PHONY: standalone
standalone:
ifeq ($(OS),Windows_NT)
	powershell.exe -NoProfile -ExecutionPolicy Bypass -File build.ps1 -Make "$(MAKE)" -Nugget "$(NUGGET_DIR)" -Build "$(BUILD)"
else
	sh build.sh "$(MAKE)" "$(NUGGET_DIR)" "$(BUILD)"
endif
endif
