use clap::*;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::net::TcpStream;
use std::process::Command;
use chrono::{DateTime,Local};
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskInfo {
    pub name: String,
    pub priority: i32,
    pub core_num: i32,
    pub username: Option<String>,
    pub base_dir: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum TaskStatus {
    Wait,
    Doing,
    Done,
    Error,
}
impl std::fmt::Display for TaskStatus{
    
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> { 
    write!(f,"{:?}",self)
 }
}
#[derive(Serialize, Deserialize, Debug)]
struct QueryListCMD {
    pub from: i32,
    pub to: i32,
    pub status: Option<TaskStatus>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SingleCMD {
    pub name: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ServerStatus {
    pub  core_num: i32,
    pub  task_num: i32,
    pub  current_task_num: i32,
    pub runing_core: i32,
}


#[derive(Debug,Serialize, Deserialize)]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub priority: i32,
    pub base_dir: String,
    pub status: TaskStatus,
    pub core_num: i32,
    pub created_at: DateTime<Local>,
    pub finished_at: Option<DateTime<Local>>,
    pub username:String,
    pub msg :Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
#[allow(clippy::enum_variant_names)]
#[serde(untagged)]
pub enum HanderResult {
    None,
    TaskList(Vec<Task>),
    Task(Task),
    ServerStatus(ServerStatus),
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Reponse {
    code: i32,
    msg: String,
    data: HanderResult,
}

impl Reponse{
    fn print<P: AsRef<str>>(&self,message:P){
        if self.code !=0{
            println!("{}",self.msg);
            return;
        }
    
        match &self.data{
            HanderResult::None=>println!("{}",message.as_ref()),
            HanderResult::Task(task)=>{
                println!(
                    "id          : {}\ntask_name   : {}\nusername    : {}\ncore_num    : {}\nbase_dir    : {}\nstatus      : {}\ncreated_at  : {}\nfinished_at : {:?}\nmsg         : {:?}",
                    task.id,task.name,task.username,task.core_num,task.base_dir,task.status,task.created_at,task.finished_at,task.msg
                );

                ()
            },
            HanderResult::TaskList(list)=>{
                       println!(

                    "{0: <5} | {1: <10} | {2: <10} | {3: <20} | {4:<8} |{5:<20} |",
                    "id", "task_name","username","base_dir","status","msg"
                );
                for item in list {
                    println!(
                        "{0: <5} | {1: <10} | {2: <10} | {3: <20} | {4:<8?} |{5:<20?} |",
                        item.id, &item.name,&item.username,&item.base_dir,&item.status,&item.msg
                    );
                }
            },
            HanderResult::ServerStatus(status)=>println!("{:?}",status),
        }
    }
}

fn send_command(port: i32, data: Vec<u8>)->Reponse {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).expect("port error");
    stream.write(&data.as_slice()).expect("send error");
    let mut buf = [0; 4096];
    let sie = stream.read(&mut buf).unwrap();
    let d = std::str::from_utf8(&buf[0..sie]).unwrap();
    serde_json::from_slice(&buf[0..sie]).unwrap()
}

fn create(matches: &ArgMatches) -> Vec<u8> {
    let output = Command::new("whoami").output().unwrap();
    let temp = String::from_utf8(output.stdout).unwrap();
    let name = temp.trim();
    let mut base_dir = std::path::PathBuf::from(std::env::current_dir().unwrap());
    let bd = matches.value_of("base_dir").unwrap();
    base_dir.push(bd);
    let mut yaml_path = std::path::PathBuf::from(&base_dir);
    yaml_path.push("task.yaml");
    let data = std::fs::read(&yaml_path).expect(&format!("cannot read {}", base_dir.display()));
    let mut info: TaskInfo = serde_yaml::from_slice(data.as_slice()).expect("read yaml file error");
    info.base_dir = Some(base_dir.to_str().unwrap().to_string());
    info.username = Some(name.to_string());
    let mut result: Vec<u8> = Vec::new();
    result.extend("create".as_bytes());
    result.push(3);
    let json = serde_json::to_vec(&info).unwrap();
    result.extend(json);
    return result;
}

fn list(matches: &ArgMatches) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    result.extend("query_list".as_bytes());
    result.push(3);
    let from = matches.value_of("start").unwrap().parse().unwrap();
    let to= matches.value_of("end").unwrap().parse().unwrap();
    let status= if  matches.is_present("status") {
        Some(TaskStatus::Doing)
    }else{
        None
    };
    let info =QueryListCMD {
         from,
         to,
         status
    };
    let json = serde_json::to_vec(&info).unwrap();
    result.extend(json);
    return result;
}

fn server(matches: &ArgMatches) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    result.extend("status".as_bytes());
    result.push(3);
    return result;
}
fn single<P:AsRef<str>>(matches: &ArgMatches,cmd:P) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    result.extend(cmd.as_ref().as_bytes());
    result.push(3);
    let name= matches.value_of("name").unwrap().to_string();
    println!("{}",&name);
    let info= SingleCMD{name};
    let json = serde_json::to_vec(&info).unwrap();
    result.extend(json);
    return result;
}
fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        let msg = match panic_info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match panic_info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };
        println!("Exit by {}", msg);

        // print!("panic info: {:?}",panic_info.message.);
    }));
    let def_base_dir: String = format!("{}", std::env::current_dir().unwrap().display());
    let matches = App::new("osier-cli")
        .arg(
            Arg::with_name("port")
                .default_value("1115")
                .help("port")
                .long("port"),
        )
        .subcommand(App::new("create").arg(Arg::with_name("base_dir").default_value(&def_base_dir)))
        .subcommand(App::new("status").arg(Arg::with_name("name").empty_values(false).required(true).long("name")))
        .subcommand(App::new("server").arg(Arg::with_name("status").long("status").empty_values(true)))
        .subcommand(
            App::new("list")
                .arg(Arg::with_name("start").default_value("0").long("start"))
                .arg(Arg::with_name("status").long("status"))
                .arg(Arg::with_name("end").default_value("10").long("end")),
        )
        .subcommand(App::new("restart").arg(Arg::with_name("name").empty_values(false).required(true).long("name")))
        .subcommand(App::new("remove").arg(Arg::with_name("name").empty_values(false).required(true).long("name")))
        .get_matches();
    let port = matches.value_of("port").unwrap();
    let (data,success_msg) = match matches.subcommand() {
        ("create", Some(create_matches)) => {
            // Now we have a reference to clone's matches
            (create(create_matches),"create success")
        },
        ("list",Some(list_matches))=>{
            (list(list_matches),"")
        },
        ("server",Some(server_matches))=>{
            ( server(server_matches),"")
        },
        ("remove",Some(single_matches))=>{
            ( single(single_matches,"remove"),"remove success")
        },
        ("restart",Some(single_matches))=>{
            ( single(single_matches,"restart"),"restart success")
        },
        ("status",Some(single_matches))=>{
            ( single(single_matches,"query"),"")
        },
        ("", None) => {
            println!("No subcommand was used");
            return;
        } // If no subcommand was usd it'll match the tuple ("", None)
        _ => {
            println!("No subcommand was used");
            return;
        }
    };
    let response = send_command(port.parse().unwrap(), data);
    response.print(success_msg);

}
