import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.util.PseudoDisassembler;

/** Read-only camera setup / trigonometry audit for SCUS_942.44. */
public class OriginalCameraMathAudit extends GhidraScript {
    public void run() throws Exception {
        DecompInterface decompiler = new DecompInterface();
        try {
            decompiler.openProgram(currentProgram);
            for (long address : new long[]{0x80018a54L, 0x80018988L}) {
                var function = getFunctionAt(toAddr(address));
                if (function == null) throw new IllegalStateException("Missing function: " + Long.toHexString(address));
                var result = decompiler.decompileFunction(function, 60, monitor);
                if (!result.decompileCompleted()) throw new IllegalStateException(result.getErrorMessage());
                println(result.getDecompiledFunction().getC());
            }
        } finally { decompiler.dispose(); }
        PseudoDisassembler disassembler = new PseudoDisassembler(currentProgram);
        long[][] ranges = {
            {0x80011ab8L, 0x80011b04L}, // Caller uploads R, not the world callback.
            {0x8003c2f0L, 0x8003c3d8L}, // Sine/cosine quadrant lookup.
            {0x8003f518L, 0x8003f520L}, // Empty setup callback.
            {0x80041e1cL, 0x80041e58L}, // Effect/scratchpad setup callback.
            {0x8004f674L, 0x8004f6a8L}  // SetRotMatrix control-register upload.
        };
        for (long[] range : ranges)
            for (long address = range[0]; address < range[1]; address += 4)
                println(Long.toHexString(address) + " " + disassembler.disassemble(toAddr(address)));
        int mismatches = 0, hash = 0x811c9dc5;
        for (int i = 0; i <= 1024; ++i) {
            int actual = getShort(toAddr(0x8005c91cL + 2 * i));
            int expected = (int)Math.round(Math.sin(i * Math.PI / 2048.0) * 4096.0);
            if (actual != expected) ++mismatches;
            hash = (hash ^ (actual & 255)) * 0x01000193;
            hash = (hash ^ ((actual >>> 8) & 255)) * 0x01000193;
        }
        println(String.format("Quarter table: 1025 entries / 2050 bytes; mathematical mismatches=%d, FNV=%08x", mismatches, hash));
    }
}
