//! Standalone harness for the production Rust codec. Never embeds game data.
//! Input EPOKSEQ1: u32 sample_count, then u16 count/reset and count*u32 tokens.
//! Output EPOKORD1: u32 samples/max_count, then 12-byte spans and encoded bytes.
#[path="../src/draw_order_codec.rs"] mod draw_order_codec;
use std::{fs,io::Write};
fn main() -> Result<(),Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len()!=3{return Err("Usage: draw_order_probe input.seq new-output.bin".into());}
    let raw=fs::read(&args[1])?;
    if raw.get(..8)!=Some(b"EPOKSEQ1"){return Err("Unknown sequence input".into());}
    let mut cursor=8;
    fn read<const N:usize>(data:&[u8],cursor:&mut usize)->Result<[u8;N],Box<dyn std::error::Error>>{
        let result=data.get(*cursor..*cursor+N).ok_or("Truncated sequence")?.try_into()?;*cursor+=N;Ok(result)
    }
    let count=u32::from_le_bytes(read(&raw,&mut cursor)?) as usize;
    if count>65535{return Err("Too many sequence samples".into());}
    let mut lists=Vec::new();
    for _ in 0..count {
        let n=u16::from_le_bytes(read(&raw,&mut cursor)?) as usize;
        let reset=u16::from_le_bytes(read(&raw,&mut cursor)?);
        if reset>1{return Err("Invalid reset flag".into());}
        let mut values=Vec::new();
        for _ in 0..n { values.push(u32::from_le_bytes(read(&raw,&mut cursor)?)); }
        lists.push((values,reset!=0));
    }
    if cursor!=raw.len(){return Err("Trailing sequence data".into());}
    let (spans,data)=draw_order_codec::pack(lists.iter().map(|(v,reset)|(v.as_slice(),*reset)))?;
    let maximum=lists.iter().map(|(v,_)|v.len()).max().unwrap_or(0);
    let mut output=fs::OpenOptions::new().write(true).create_new(true).open(&args[2])?;
    output.write_all(b"EPOKORD1")?;
    output.write_all(&(count as u32).to_le_bytes())?;
    output.write_all(&(maximum as u32).to_le_bytes())?;
    for span in &spans {
        output.write_all(&span.offset.to_le_bytes())?;output.write_all(&span.bytes.to_le_bytes())?;
        output.write_all(&span.count.to_le_bytes())?;output.write_all(&u16::from(span.reset).to_le_bytes())?;
    }
    output.write_all(&data)?;
    println!("Ordered draw: {} samples, {} encoded bytes, {} metadata bytes, {} bytes for two decode lists",count,data.len(),spans.len()*12,maximum*8);
    Ok(())
}
