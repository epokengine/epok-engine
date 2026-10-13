import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.util.PseudoDisassembler;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.charset.StandardCharsets;
import java.nio.file.StandardOpenOption;

/** Read-only function audit for the original rail/target camera, SCUS_942.44. */
public class OriginalCameraFollowAudit extends GhidraScript {
    public void run() throws Exception {
        if (getScriptArgs().length != 1) throw new IllegalArgumentException("New output directory required");
        Path directory = Path.of(getScriptArgs()[0]);
        if (Files.exists(directory)) throw new IllegalArgumentException("Output already exists");
        Files.createDirectories(directory);
        StringBuilder manifest = new StringBuilder("Program: " + currentProgram.getName()
            + "\nExecutable SHA256: " + currentProgram.getExecutableSHA256() + "\n");
        DecompInterface decompiler = new DecompInterface();
        try {
            decompiler.openProgram(currentProgram);
            for (long address : new long[]{0x80025188L, 0x80031e84L, 0x80031e10L,
                    0x80031f9cL, 0x80031e34L, 0x80025c54L, 0x800233dcL,
                    0x800215e8L, 0x80028a78L, 0x8002437cL, 0x800247dcL, 0x8004f794L}) {
                var function = getFunctionAt(toAddr(address));
                if (function == null) throw new IllegalStateException("Missing function " + Long.toHexString(address));
                var result = decompiler.decompileFunction(function, 60, monitor);
                if (!result.decompileCompleted()) throw new IllegalStateException(result.getErrorMessage());
                String code = result.getDecompiledFunction().getC();
                Path path = directory.resolve(Long.toHexString(address) + ".c");
                Files.writeString(path, code, StandardCharsets.UTF_8, StandardOpenOption.CREATE_NEW);
                String description = Long.toHexString(address) + ".." + function.getBody().getMaxAddress()
                    + " " + function.getName() + " " + code.length() + " characters\n";
                manifest.append(description);
                println(description.trim());
            }
        } finally { decompiler.dispose(); }
        StringBuilder assembly = new StringBuilder();
        PseudoDisassembler disassembler = new PseudoDisassembler(currentProgram);
        for (long[] range : new long[][]{{0x80025188L, 0x800251ecL}, {0x80025570L, 0x800255ccL},
                {0x80031e10L, 0x80031e84L}, {0x80031f9cL, 0x80031fb4L},
                {0x8004f794L, 0x8004f91cL}})
            for (long a = range[0]; a < range[1]; a += 4)
                assembly.append(Long.toHexString(a)).append(' ').append(disassembler.disassemble(toAddr(a))).append('\n');
        Files.writeString(directory.resolve("boundaries.asm"), assembly.toString(), StandardCharsets.UTF_8, StandardOpenOption.CREATE_NEW);
        int rounded = 0, floored = 0;
        for (int i = 0; i <= 1024; ++i) {
            int actual = getShort(toAddr(0x8005f6b8L + i * 2));
            double angle = Math.atan(i / 1024.0) * 4096.0 / (2.0 * Math.PI);
            if (actual != (int)Math.round(angle)) ++rounded;
            if (actual != (int)Math.floor(angle)) ++floored;
        }
        manifest.append("Atan quarter-ratio table 8005f6b8: 1025 entries; round mismatches=")
            .append(rounded).append("; floor mismatches=").append(floored).append('\n');
        Files.writeString(directory.resolve("manifest.txt"), manifest.toString(), StandardCharsets.UTF_8, StandardOpenOption.CREATE_NEW);
        println("Saved read-only audit to " + directory);
    }
}
