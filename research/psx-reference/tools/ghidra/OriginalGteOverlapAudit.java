import ghidra.app.script.GhidraScript;
import ghidra.app.util.PseudoDisassembler;

/** Read-only RTPT / independent CPU work / result-consumption audit. */
public class OriginalGteOverlapAudit extends GhidraScript {
    public void run() throws Exception {
        PseudoDisassembler disassembler = new PseudoDisassembler(currentProgram);
        for (long address = 0x80041640L; address < 0x80041790L; address += 4)
            println(Long.toHexString(address) + " " + disassembler.disassemble(toAddr(address)));
    }
}
