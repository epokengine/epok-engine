import ghidra.app.script.GhidraScript;
import ghidra.app.util.PseudoDisassembler;

/** Read-only packed screen rejection audit, SCUS_942.44. */
public class OriginalScreenRejectAudit extends GhidraScript {
    public void run() throws Exception {
        PseudoDisassembler disassembler = new PseudoDisassembler(currentProgram);
        for (long address=0x800416d4L; address<0x80041718L; address+=4)
            println(Long.toHexString(address)+" "+disassembler.disassemble(toAddr(address)));
    }
}
