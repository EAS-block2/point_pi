//Digital Signage and Strobe systems
use std::net::{TcpListener};
use std::io::{Read, Write};
use std::time::Duration;
use std::{str, thread};
use gpio::GpioOut;
use crossbeam_channel::unbounded;
fn main() {
    let mut general = Alarm {render_name: "General".to_string(), pin:25, active: false};
    let mut silent = Alarm {render_name: "Silent".to_string(), pin:28, active: false};
    let mut locations: Vec<String> = vec!();
    //let alarms = vec!(general, silent);
    let (threadcom_s, threadcom_r) = unbounded();
    println!("starting");
    let listener = TcpListener::bind("192.168.1.149:5400").unwrap();
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(mut streamm) => {
                    let mut data = [0 as u8; 50];
                    match streamm.read(&mut data){
                        Ok(size) => {
                        match str::from_utf8(&data[0..size]){
                            Ok(string_out) => {
                                let s: String = (&string_out).to_string();
                                let msg: Vec<u8>;
                                println!("Got data: {}", s);
                                match threadcom_s.send(s){Ok(_)=> msg = b"ok".to_vec(), Err(e)=>{println!("{}",e); msg=b"fault".to_vec();}}
                                match streamm.write(&msg) {
                                Ok(_) => {println!("Write success")},
                                Err(e) => {println!("Write Error: {}", e)}
                                }}
                            Err(_) => {println!("fault"); break;}
                        }}
                        Err(_) => {println!("Fault when reading data!"); break;}
                    }}
                Err(e) => {println!("Connection failed with code {}", e);thread::sleep(Duration::from_secs(1));}
            }}
    });
    loop{
        println!("General: {}, Silent: {}", general.active, silent.active);
        println!("Locations: {:?}", locations);
        thread::sleep(Duration::from_millis(500));
        match threadcom_r.try_recv(){
            Ok(out) => {
                let e = out.split(' ');
                let mut alm = String::new();
                for i in e{
                    if alm.is_empty() {alm = i.to_string();}
                    else {locations.push(i.to_string());}
                }
                if alm.to_lowercase() == "general" {general.active = true;}
                else if alm.to_lowercase() == "silent" {silent.active = true;}
            }
            Err(_) => thread::sleep(Duration::from_secs(2)), //usually will return an error as no data has been sent
        }
    }
}
struct Alarm{
    render_name: String,
    pin: u8,
    active: bool
}
