import ghidra.app.script.GhidraScript;
import ghidra.app.util.PseudoDisassembler;
public class OrderedWorldBasisAudit extends GhidraScript {
    @Override public void run() throws Exception {
        println("READ ONLY: " + currentProgram.getDomainFile().getPathname());
        PseudoDisassembler decoder = new PseudoDisassembler(currentProgram);
        for (long pc = 0x80040ed8L; pc <= 0x800414acL; pc += 4)
            println(toAddr(pc) + " " + decoder.disassemble(toAddr(pc)));
    }
}
