// Links the engine's existing PocketStation library; no Cargo cache is created.
use epok_pocketstation::{card,file::{Header,ColorIcon,MenuIcon},plua,Machine};
fn variable(p:&plua::Program,name:&str)->u32{p.variables.iter().find(|v|v.0==name).unwrap_or_else(||panic!("Missing variable {name}")).1}
fn values(m:&mut Machine)->[u32;16]{
    let mut f=vec![0x81,0x5b,0x82,0,0];f.extend([0;65]);let r=m.link_transfer(&f);assert_eq!(r[4],64);
    std::array::from_fn(|i|u32::from_le_bytes(r[5+i*4..9+i*4].try_into().unwrap()))
}
fn ack(m:&mut Machine,n:u32){
    let mut f=vec![0x81,0x5c,0x80,0,5,0,0xd1];f.extend(n.to_le_bytes());f.push(0);
    let r=m.link_transfer(&f);assert_eq!(r[5],5);m.run(0.15);
}
fn main(){
    let args:Vec<String>=std::env::args().collect();
    if args.get(1).map(String::as_str)==Some("--blank"){
        std::fs::write(&args[2],card::formatted()).unwrap();println!("Created formatted PocketStation without installed applications.");return;
    }
    if args.get(1).map(String::as_str)==Some("--check-installed"){
        let flash=std::fs::read(&args[2]).unwrap();let file=card::find(&flash,"BEEPOKPASTEROID").expect("Installed file");
        let mut m=Machine::new(flash);m.set_docked(true);m.launch(file.block,0).unwrap();m.run(0.3);
        let v=values(&mut m);assert_eq!(v[2],0x41535431);assert_eq!(v[4],1);assert_eq!(v[0],0);
        println!("PASS: PSX installed a launchable Asteroid Catcher program onto an empty PocketStation.");return;
    }
    let source=std::fs::read_to_string(std::env::var("EPOK_DUNGEON_MINER_SOURCE").expect("Set the authored miner source")).unwrap();
    let mut h=Header{title:"Asteroid Catcher".into(),color_icon:ColorIcon::default(),viewer_icons:vec![],
        menu_icons:vec![MenuIcon{frames:vec![[0;32]],delay:8}],function_count:0,snapshot:false};
    let p=plua::compile(&source,&mut h,&plua::Options::default()).unwrap_or_else(|e|panic!("{e:#?}"));
    let mut bytes=plua::executable(&h,&p).unwrap();assert!(p.uses_link&&p.uses_save);assert!(p.stack_bytes()>=512);
    if let Ok(path)=std::env::var("EPOK_DUNGEON_MINER_IMAGE"){
        let actual=std::fs::read(path).unwrap();
        assert_eq!(&actual[h.layout().body..],&bytes[h.layout().body..],"Staged ARM code must match authored source");
        bytes=actual;
    }
    let deployed=args.get(1).map(String::as_str)==Some("--image");
    if deployed{
        let actual=std::fs::read(&args[2]).unwrap();
        assert_eq!(&actual[h.layout().body..],&bytes[h.layout().body..],"Staged ARM code must match the authored source");
        bytes=actual;
    }
    assert_eq!(bytes.len(),8192,"The exit must still fit in one card block");
    if args.get(1).map(String::as_str)==Some("--export"){
        std::fs::write(&args[2],&bytes).unwrap();
        let offset=p.assembly.labels["rt_save_sector_data"]-0x02000000;
        std::fs::write(std::path::Path::new(&args[2]).with_extension("save-offset"),offset.to_string()).unwrap();
        println!("save_sector={}",offset/128);return;
    }
    let mut flash=card::formatted();let block=card::write(&mut flash,"BEEPOKPASTEROID",&bytes).unwrap();
    let mut m=Machine::new(flash);m.set_docked(true);m.launch(block,0).unwrap();m.run(0.2);
    assert_eq!(values(&mut m)[2],0x41535431);
    // Start a real round, then arrange a catch at the collision boundary.
    m.set_buttons(epok_pocketstation::hw::button::FIRE);m.run(0.05);m.set_buttons(0);m.run(0.04);
    m.hw.set_ram_word(variable(&p,"kind")+4,1);m.hw.set_ram_word(variable(&p,"fall_x")+4,12);
    m.hw.set_ram_word(variable(&p,"fall_y")+4,25);m.hw.set_ram_word(variable(&p,"timer"),5);
    m.hw.set_ram_word(variable(&p,"speed"),6);m.hw.set_ram_word(variable(&p,"player"),12);m.run(0.06);
    assert_eq!(values(&mut m)[0],1,"A caught asteroid must persist as ore");
    if let Some(path)=std::env::args().nth(1).filter(|_|!deployed){
        for _ in 1..40{
            for i in 0..6{m.hw.set_ram_word(variable(&p,"kind")+4+i*4,0);}
            m.hw.set_ram_word(variable(&p,"kind")+4,1);m.hw.set_ram_word(variable(&p,"fall_x")+4,12);
            m.hw.set_ram_word(variable(&p,"fall_y")+4,25);m.hw.set_ram_word(variable(&p,"timer"),5);m.run(0.06);
        }
        assert_eq!(values(&mut m)[0],40);std::fs::write(path,&m.hw.flash).unwrap();
        println!("Created validation card from 40 real ARM catches.");return;
    }
    ack(&mut m,1);assert_eq!(values(&mut m)[3],1);
    ack(&mut m,1);assert_eq!(values(&mut m)[3],1,"Duplicate receipt must be idempotent");
    ack(&mut m,2);assert_eq!(values(&mut m)[3],1,"Cannot acknowledge uncollected ore");
    let mut restarted=Machine::new(m.hw.flash.clone());restarted.set_docked(true);restarted.launch(block,0).unwrap();restarted.run(0.2);
    let v=values(&mut restarted);assert_eq!(v[0],1);assert_eq!(v[3],1);
    // Partial holds cancel, and holding through five seconds pauses the game.
    restarted.set_buttons(epok_pocketstation::hw::button::FIRE);restarted.run(1.0);
    let timer=restarted.hw.ram_word(variable(&p,"timer"));restarted.run(1.0);
    assert_eq!(restarted.hw.ram_word(variable(&p,"timer")),timer);
    restarted.set_buttons(0);restarted.run(0.1);
    assert!(!matches!(restarted.state,epok_pocketstation::State::Launcher));
    restarted.set_date_time(epok_pocketstation::DateTime{year:2026,month:10,day:22,hour:12,minute:0,second:10,weekday:5});
    restarted.hw.set_ram_word(variable(&p,"score"),23);
    restarted.set_buttons(epok_pocketstation::hw::button::FIRE);restarted.run(5.2);
    assert!(!matches!(restarted.state,epok_pocketstation::State::Launcher),"Wait for release before entering the GUI");
    let date=restarted.date_time();restarted.set_buttons(0);restarted.run(0.2);
    assert!(matches!(restarted.state,epok_pocketstation::State::Launcher));
    assert_eq!(restarted.date_time().year,date.year);assert_eq!(restarted.date_time().month,date.month);
    assert_eq!(restarted.date_time().day,date.day);assert_eq!(restarted.date_time().hour,date.hour);
    restarted.run(0.5);assert!(matches!(restarted.state,epok_pocketstation::State::Launcher));
    // The PSX cannot call an app's values function after exit. Its transfer
    // command must first remotely start the installed copy, then wait for init.
    assert_eq!(restarted.link_transfer(&[0x81,0x5b,0x82,0,0,0]).len(),3);
    let start=[0x81,0x59,0,0,block,0,0,0,0];
    let reply=restarted.link_transfer(&start);assert_eq!(reply.len(),9);assert_eq!(reply[2],6);
    restarted.run(0.2);assert!(matches!(restarted.state,epok_pocketstation::State::Program(b) if b==block));
    let v=values(&mut restarted);assert_eq!(v[0],1);assert_eq!(v[1],23);assert_eq!(v[3],1);assert_eq!(v[4],1);
    println!("PASS: ARM catch/save/receipts/restart, canceled hold, five-second save-and-exit, RTC preservation and PSX remote relaunch. {} bytes, {} stack bytes.",bytes.len(),p.stack_bytes());
}
