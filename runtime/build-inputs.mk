# Host-only inspection target. Normal standalone builds do not invoke it.
# GCC's .dep files supply the actual transitive C/C++ include closure.
# Match the built-in object recipes' flag order, including target-specific flags.
%.dep: %.cpp
	$(COMPILE.cpp) -M -MT $(patsubst %.dep,%.o,$@) -MF $@ $<
%.dep: %.cc
	$(COMPILE.cc) -M -MT $(patsubst %.dep,%.o,$@) -MF $@ $<
%.dep: %.c
	$(COMPILE.c) -M -MT $(patsubst %.dep,%.o,$@) -MF $@ $<

ifeq ($(TYPE),library)
# The assembler reports .include/.incbin inputs while assembling a disposable
# probe object. Do not overwrite an object that a normal SDK build could reuse.
%.dep: %.s
	$(CC) $(ARCHFLAGS) -I$(ROOTDIR) -g -c -o $@.epok-probe.o -Wa,--MD,$@ $<
ifneq ($(EPOK_SDK_OBJECT_DIRECTORY),)
# Keep the original object targets so their target-specific flags still apply,
# but put every compiled object in this invocation's private directory. The host
# invokes -B: original SDK object timestamps cannot bypass these recipes.
EPOK_PRIVATE_OBJECTS = $(foreach o,$(OBJS),$(EPOK_SDK_OBJECT_DIRECTORY)/$(subst /,_,$(o)))
ifneq ($(words $(EPOK_PRIVATE_OBJECTS)),$(words $(sort $(EPOK_PRIVATE_OBJECTS))))
$(error SDK object names collide in the private build directory)
endif
%.o: %.cpp
	$(COMPILE.cpp) -o "$(EPOK_SDK_OBJECT_DIRECTORY)/$(subst /,_,$@)" $<
%.o: %.cc
	$(COMPILE.cc) -o "$(EPOK_SDK_OBJECT_DIRECTORY)/$(subst /,_,$@)" $<
%.o: %.c
	$(COMPILE.c) -o "$(EPOK_SDK_OBJECT_DIRECTORY)/$(subst /,_,$@)" $<
%.o: %.s
	$(CC) $(ARCHFLAGS) -I$(ROOTDIR) -g -c -o "$(EPOK_SDK_OBJECT_DIRECTORY)/$(subst /,_,$@)" $<
endif
.PHONY: epok-sdk-archive
epok-sdk-archive: $(OBJS)
	$(if $(EPOK_SDK_OBJECT_DIRECTORY),,$(error SDK archive build requires a private object directory))
	$(AR) rcs "$(EPOK_SDK_ARCHIVE_OUTPUT)" $(EPOK_PRIVATE_OBJECTS)
else
# Use a certified archive owned by this application build when the host supplies
# one. Standalone exports retain Nugget's ordinary SDK library build.
ifneq ($(EPOK_CERTIFIED_SDK),)
LIBRARIES := $(EPOK_CERTIFIED_SDK) $(filter-out $(PSYQODIR)libpsyqo.a,$(LIBRARIES))
endif
.PHONY: epok-standalone-sdk
epok-standalone-sdk:
	$(MAKE) -C "$(PSYQODIR)" -f Makefile -f "$(abspath build-inputs.mk)" -B epok-sdk-archive BUILD=$(BUILD) CPPFLAGS_$(BUILD)="$(CPPFLAGS_$(BUILD))" LDFLAGS_$(BUILD)="$(LDFLAGS_$(BUILD))" EPOK_SDK_OBJECT_DIRECTORY="$(EPOK_SDK_OBJECT_DIRECTORY)" EPOK_SDK_ARCHIVE_OUTPUT="$(abspath $(EPOK_CERTIFIED_SDK))"
endif

# Use info rather than file so inspection also works with GNU Make 3.81.
# The host captures these records without invoking a platform-specific shell.
.PHONY: epok-build-inputs
epok-build-inputs: dep
	$(info EPOK_NATIVE_INPUT:epok-build-inputs-v1)
	$(foreach v,CC CXX AR SHELL,$(info EPOK_NATIVE_INPUT:tool:$(v):$($(v))))
	$(info EPOK_NATIVE_INPUT:tool:OBJCOPY:$(PREFIX)-objcopy)
	$(info EPOK_NATIVE_INPUT:value:SHELL_ORIGIN:$(origin SHELL))
	$(foreach v,BUILD TYPE PREFIX ARCHFLAGS CPPFLAGS CXXFLAGS CFLAGS LDFLAGS CPPFLAGS_Release LDFLAGS_Release COMPILE.cpp COMPILE.cc COMPILE.c TARGET_ARCH EPOK_RUNTIME_OPT PSYQODIR,$(info EPOK_NATIVE_INPUT:value:$(v):$($(v))))
	$(foreach f,$(filter-out $(DEPS),$(MAKEFILE_LIST)) $(SRCS) $(if $(filter library,$(TYPE)),,$(LDSCRIPTS)),$(info EPOK_NATIVE_INPUT:file:$(f)))
	$(foreach f,$(DEPS),$(info EPOK_NATIVE_INPUT:dependency:$(f)))
	$(foreach f,$(LIBRARIES),$(info EPOK_NATIVE_INPUT:library:$(f)))

# The editor supplies this stamp only when certifying a native build. -W forces
# application recompilation after changed inputs or an unsuccessful prior build,
# including changes that preserve filesystem timestamps.
ifneq ($(EPOK_INPUT_STAMP),)
$(OBJS) $(BINDIR)$(TARGET).elf: $(EPOK_INPUT_STAMP)
endif
