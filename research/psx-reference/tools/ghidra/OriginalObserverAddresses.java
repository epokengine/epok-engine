import ghidra.app.script.GhidraScript;
import ghidra.app.util.PseudoDisassembler;

/** Read-only audit of the original runtime observer's boundaries. */
public class OriginalObserverAddresses extends GhidraScript {
    public void run() throws Exception {
        PseudoDisassembler disassembler = new PseudoDisassembler(currentProgram);
        long[][] ranges = {
            {0x80040ed8L, 0x80040f50L},
            {0x8004150cL, 0x80041550L},
            {0x800169d0L, 0x80016ce4L},
            {0x8004ce30L, 0x8004cea0L}
        };
        for (long[] range : ranges) {
            for (long address = range[0]; address < range[1]; address += 4) {
                println(Long.toHexString(address) + " " + disassembler.disassemble(toAddr(address)));
            }
        }
    }
}
