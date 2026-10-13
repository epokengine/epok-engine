import ghidra.app.script.GhidraScript;
import ghidra.app.util.PseudoDisassembler;
public class OriginalVertexInputAudit extends GhidraScript {
    @Override public void run() throws Exception {
        println("READ ONLY: " + currentProgram.getDomainFile().getPathname());
        PseudoDisassembler decoder = new PseudoDisassembler(currentProgram);
        for (long pc = 0x800410ecL; pc <= 0x800412acL; pc += 4)
            println(toAddr(pc) + " " + decoder.disassemble(toAddr(pc)));
    }
}
