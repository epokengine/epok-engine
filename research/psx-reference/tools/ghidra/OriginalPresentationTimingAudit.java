import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.util.PseudoDisassembler;

/** Read-only frame pacing, DrawSync, timer setup and display-origin audit. */
public class OriginalPresentationTimingAudit extends GhidraScript {
    public void run() throws Exception {
        DecompInterface decompiler = new DecompInterface();
        try {
            decompiler.openProgram(currentProgram);
            for (long address : new long[] {0x80016634L, 0x8004b2a8L, 0x8004c600L, 0x8004ea10L}) {
                var function = getFunctionAt(toAddr(address));
                if (function == null) throw new IllegalStateException("Missing function at " + Long.toHexString(address));
                var result = decompiler.decompileFunction(function, 60, monitor);
                if (!result.decompileCompleted()) throw new IllegalStateException(result.getErrorMessage());
                println(result.getDecompiledFunction().getC());
            }
        } finally {
            decompiler.dispose();
        }
        PseudoDisassembler disassembler = new PseudoDisassembler(currentProgram);
        long[][] ranges = {
            {0x80015598L, 0x800155f8L}, // timer 2 target 0x1010, flags 0x1000
            {0x8003ae70L, 0x8003ae8cL}, // timer callback increments the pacing clock
            {0x80017ad0L, 0x80017ae8L}, // vblank callback increments its own counter
            {0x80048cd8L, 0x80048d74L}, // SetRCnt maps timer 2 flags to mode 0x258
            {0x80016a10L, 0x80016ae0L}, // DrawSync, buffer rotation and conditional VSync waits
            {0x8004ea7cL, 0x8004eac4L}, // DrawSync DMA bit 24 and GPUSTAT bit 26
            {0x8004ce84L, 0x8004cec0L}  // display-origin command construction
        };
        for (long[] range : ranges)
            for (long address = range[0]; address < range[1]; address += 4)
                println(Long.toHexString(address) + " " + disassembler.disassemble(toAddr(address)));
        long table = Integer.toUnsignedLong(getInt(toAddr(0x8005f400L)));
        println("DrawSync table = " + Long.toHexString(table));
        println("DrawSync target = " + Integer.toHexString(getInt(toAddr(table + 0x3c))));
        println("DMA register pointer = " + Integer.toHexString(getInt(toAddr(0x8005f4ecL))));
        println("GPU register pointer = " + Integer.toHexString(getInt(toAddr(0x8005f4e0L))));
    }
}
