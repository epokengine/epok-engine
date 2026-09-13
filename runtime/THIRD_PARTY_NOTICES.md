# Runtime third-party notices

The accompanying Epok runtime sources are licensed under the included Epok MIT license. This license does not assign a license to user-authored game scripts or assets.

The supported SDK is Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`:
https://github.com/pcsx-redux/nugget/tree/6186b131aacc5853a9161fb076ed34ffe504552d

| Component | Attribution | License text |
| --- | --- | --- |
| Nugget/PsyQo and common runtime | PCSX-Redux authors | [MIT](licenses/nugget-MIT.txt) |
| EASTL | Electronic Arts and contributors | [BSD 3-Clause](licenses/EASTL.txt), [additional notices](licenses/EASTL-third-party.txt) |
| EABase | Electronic Arts and contributors | [BSD 3-Clause](licenses/EABase.txt) |
| GCC libgcc integer arithmetic helpers | Free Software Foundation, Inc. and contributors | [GPL version 3 or later](licenses/GCC-GPL-3.txt), with [GCC Runtime Library Exception 3.1](licenses/GCC-Runtime-Exception-3.1.txt) |
| mig68000 system font | Zingot Games | [Font attribution](licenses/psx-font.txt) |

The system font is included by PsyQo. Its pinned source credits Zingot Games and permits free use with attribution. The author's FontPack page lists CC BY 4.0:
https://zingot.itch.io/fontpack

Author website: https://www.zingot.com/
License: https://creativecommons.org/licenses/by/4.0/
PsyQo font source: https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/src/system-font.inc

Preserve these notices with redistributed builds. Changing the SDK or adding libraries/assets requires reviewing their own notices. The MIPS compiler distribution includes its own runtime-library licensing; preserve any additional notices applicable to libraries linked by your build.

The exported Makefile links the MIPS compiler's `libgcc.a` for 64-bit integer division used by collision, clipping and measured time. The GCC runtime source applies the Runtime Library Exception: [libgcc2.c](https://github.com/gcc-mirror/gcc/blob/master/libgcc/libgcc2.c), [exception text](https://github.com/gcc-mirror/gcc/blob/master/COPYING.RUNTIME). The exception permits eligible compiled programs to use the covered runtime with independently chosen licensing for their own modules.

The standalone game does not contain the desktop editor's icon fonts, its Rust libraries, the Lua adapter or the PCSX-Redux emulator.
