import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.util.PseudoDisassembler;

/** Read-only pacing and GP1 presentation audit, SCUS_942.44. */
public class OriginalPresentationAudit extends GhidraScript {
    public void run() throws Exception {
        DecompInterface decompiler = new DecompInterface();
        try {
            decompiler.openProgram(currentProgram);
            var result = decompiler.decompileFunction(getFunctionAt(toAddr(0x80016634L)), 60, monitor);
            if (!result.decompileCompleted()) throw new IllegalStateException(result.getErrorMessage());
            println(result.getDecompiledFunction().getC());
        } finally { decompiler.dispose(); }
        PseudoDisassembler disassembler = new PseudoDisassembler(currentProgram);
        long[][] ranges = {
            {0x80016a24L, 0x80016b24L},
            {0x8004ce70L, 0x8004cec0L}
        };
        for (long[] range : ranges)
            for (long address = range[0]; address < range[1]; address += 4)
                println(Long.toHexString(address) + " " + disassembler.disassemble(toAddr(address)));
    }
}
