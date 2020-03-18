use std::io::prelude::*;
use std::net::TcpStream;
use clap::*;
use serde::{Serialize, Deserialize};
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskInfo{
   pub name: String,
   pub priority:i32,
   pub core_num:i32,
   pub base_dir:Option<String>,
}

fn send_command(port:i32,data:Vec<u8>){
 let mut stream = TcpStream::connect(format!("127.0.0.1:{}",port)).expect("port error");
    stream.write(&data.as_slice()).expect("send error");
    let mut buf = [0; 4096];
    let sie = stream.read(&mut buf).unwrap();
    let d= std::str::from_utf8(&buf[0..sie]).unwrap();
    print!("{}",d);
}

fn create(matches:&ArgMatches)->Vec<u8>{
    let mut base_dir=std::path::PathBuf::from(std::env::current_dir().unwrap());
    let bd= matches.value_of("base_dir").unwrap();
    base_dir.push(bd);
    let mut yaml_path = std::path::PathBuf::from(&base_dir);
    yaml_path.push("task.yaml");
    let data =  std::fs::read(&yaml_path).expect(&format!("cannot read {}",base_dir.display()));
    let mut info:TaskInfo = serde_yaml::from_slice(data.as_slice()).expect("read yaml file error");
    info.base_dir=Some(base_dir.to_str().unwrap().to_string());
    let mut result:Vec<u8> = Vec::new();
    result.extend("create".as_bytes());
    result.push(3);
    let json= serde_json::to_vec(&info).unwrap();
    result.extend(json);
    return result;
}

fn main() {
    std::panic::set_hook(Box::new(|panic_info|{
        let msg = match panic_info.payload().downcast_ref::<&'static str>() { 
            Some(s) => *s, 
            None => match panic_info.payload().downcast_ref::<String>() { 
                Some(s) => &s[..], 
                None => "Box<Any>", 
            } 
        }; 
            println!("Exit by {}",msg);

        // print!("panic info: {:?}",panic_info.message.);
    }));
    let def_base_dir: String = format!("{}", std::env::current_dir().unwrap().display());
   let matches= App::new("osier-cli")
    .arg(
        Arg::with_name("port")
        .default_value("1115")
        .help("port")
        .long("port")
    )
    .subcommand(
        App::new("create")
        .arg(
            Arg::with_name("base_dir")
            .default_value(&def_base_dir)
        )
    ).get_matches();
    let port= matches.value_of("port").unwrap();
    let data = match matches.subcommand(){
        ("create", Some(create_matches)) => {
            // Now we have a reference to clone's matches
            create(create_matches)
        },
        ("", None) => { println!("No subcommand was used");return ;}, // If no subcommand was usd it'll match the tuple ("", None)
        _ =>  { println!("No subcommand was used");return;},
    };
    send_command(port.parse().unwrap(),data);
}