import ghidra.app.script.GhidraScript;
import ghidra.app.util.PseudoDisassembler;
import ghidra.program.model.address.Address;
public class DumpRenderer extends GhidraScript {
 public void run() throws Exception {
  PseudoDisassembler decoder = new PseudoDisassembler(currentProgram);
  String[] args=getScriptArgs();
  long start=Long.parseUnsignedLong(args[0],16), end=Long.parseUnsignedLong(args[1],16);
  for(long p=start;p<end;p+=4){
   Address a=toAddr(p);
   try { println(a+" "+decoder.disassemble(a)); } catch(Exception e){ println(a+" ERROR "+e); }
  }
 }
}
