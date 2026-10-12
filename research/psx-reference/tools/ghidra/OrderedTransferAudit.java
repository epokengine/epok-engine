import ghidra.app.script.GhidraScript;
import ghidra.app.util.PseudoDisassembler;
public class OrderedTransferAudit extends GhidraScript {
    @Override public void run() throws Exception {
        println("READ ONLY: " + currentProgram.getDomainFile().getPathname());
        PseudoDisassembler decoder = new PseudoDisassembler(currentProgram);
        for (long pc = 0x80041610L; pc <= 0x80041658L; pc += 4)
            println(toAddr(pc) + " " + decoder.disassemble(toAddr(pc)));
        for (long pc = 0x80041778L; pc <= 0x80041780L; pc += 4)
            println(toAddr(pc) + " " + decoder.disassemble(toAddr(pc)));
    }
}
