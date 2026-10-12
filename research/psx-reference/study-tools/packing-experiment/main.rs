// Compile with rustc -O; stdin is the numeric form emitted by the audit runner.
#[path="../../../EpokEngine/src/visibility_packing.rs"] mod visibility_packing;
use std::io::{self,Read};
fn main(){
    let mut text=String::new();io::stdin().read_to_string(&mut text).unwrap();let mut values=text.split_whitespace().map(|s|s.parse::<usize>().unwrap());
    let n=values.next().unwrap();let capacity=values.next().unwrap();let target=values.next().unwrap();
    let(mut sizes,mut views,mut pages)=(Vec::new(),Vec::new(),Vec::new());
    for _ in 0..n{sizes.push(values.next().unwrap());pages.push(values.next().unwrap());let count=values.next().unwrap();views.push(values.by_ref().take(count).collect::<Vec<_>>());}
    let plan=visibility_packing::refine(&sizes,&views,&pages,capacity,target);
    for p in plan{println!("{p}");}
}
