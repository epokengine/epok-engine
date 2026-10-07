import ghidra.app.script.GhidraScript;
import ghidra.app.util.PseudoDisassembler;
public class OrderedProjectionScheduleAudit extends GhidraScript {
    @Override public void run() throws Exception {
        println("READ ONLY: " + currentProgram.getDomainFile().getPathname());
        PseudoDisassembler decoder = new PseudoDisassembler(currentProgram);
        for (long pc = 0x80041648L; pc <= 0x80041734L; pc += 4)
            println(toAddr(pc) + " " + decoder.disassemble(toAddr(pc)));
        println(toAddr(0x800419bcL) + " " + decoder.disassemble(toAddr(0x800419bcL)));
    }
}
