//Digital Signage and Strobe systems
use std::net::{TcpListener};
use std::io::{Read, Write};
use std::time::Duration;
use std::{str, thread};
use gpio::GpioOut;
use crossbeam_channel::unbounded;
fn main() {
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
        match threadcom_r.try_recv(){
            Ok(out) => {
                let e = out.split(' ');
                for i in e{
                    println!("Split elements: {}", i);
                }
            }
            Err(_) => thread::sleep(Duration::from_secs(2)), //usually will return an error as no data has been sent
        }
    }
}