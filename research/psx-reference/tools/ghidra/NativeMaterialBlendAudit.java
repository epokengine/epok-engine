import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
public class NativeMaterialBlendAudit extends GhidraScript {
 public void run() throws Exception {
  long[][] ranges={{0x8004183cL,0x800418c0L},{0x80041b14L,0x80041bc0L}};
  for(long[] r:ranges) {
   println("RANGE "+Long.toHexString(r[0])+".."+Long.toHexString(r[1]));
   for(Instruction i:currentProgram.getListing().getInstructions(toAddr(r[0]),true)) {
    if(i.getAddress().getOffset()>=r[1]) break;
    println(i.getAddress()+" "+i.toString());
   }
  }
 }
}
