import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.util.PseudoDisassembler;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.charset.StandardCharsets;

/** Read-only PSX draw-area / projection audit; optional output path must be new. */
public class OriginalViewportAudit extends GhidraScript {
    public void run() throws Exception {
        StringBuilder out = new StringBuilder();
        out.append("Program: ").append(currentProgram.getName()).append('\n');
        out.append("Executable SHA256: ").append(currentProgram.getExecutableSHA256()).append('\n');
        DecompInterface decompiler = new DecompInterface();
        try {
            decompiler.openProgram(currentProgram);
            for (long address : new long[]{0x800160d8L, 0x80018988L, 0x80018a54L}) {
                var function = getFunctionAt(toAddr(address));
                if (function == null) throw new IllegalStateException("Missing function: " + Long.toHexString(address));
                var result = decompiler.decompileFunction(function, 60, monitor);
                if (!result.decompileCompleted()) throw new IllegalStateException(result.getErrorMessage());
                out.append("\nAddress: ").append(Long.toHexString(address)).append('\n');
                out.append(result.getDecompiledFunction().getC());
            }
        } finally { decompiler.dispose(); }
        PseudoDisassembler disassembler = new PseudoDisassembler(currentProgram);
        for (long address = 0x800160d8L; address < 0x80016230L; address += 4)
            out.append(Long.toHexString(address)).append(' ')
                .append(disassembler.disassemble(toAddr(address))).append('\n');
        String[] args = getScriptArgs();
        if (args.length > 1) throw new IllegalArgumentException("Expected at most one new output path");
        if (args.length == 1) {
            Path path = Path.of(args[0]);
            Files.createDirectories(path.toAbsolutePath().getParent());
            Files.writeString(path, out.toString(), StandardCharsets.UTF_8,
                java.nio.file.StandardOpenOption.CREATE_NEW, java.nio.file.StandardOpenOption.WRITE);
            println("Saved audit: " + path + " (" + out.length() + " characters)");
        } else println(out.toString());
    }
}
