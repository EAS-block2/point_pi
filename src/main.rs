//Digital Signage and Strobe systems
use std::net::{TcpListener};
use std::io::{Read, Write};
use std::str;
fn main() {
println!("starting");
loop{
let listener = TcpListener::bind("192.168.1.149:5400").unwrap();
for stream in listener.incoming() {
    match stream {
        Ok(mut streamm) => {
            let mut data = [0 as u8; 50];
            match streamm.read(&mut data){
                Ok(size) => {
                   match str::from_utf8(&data[0..size]){
                       Ok(string_out) => {
                           println!("Got data: {}", string_out);
                           streamm.write(b"ok").unwrap();
                       }
                       Err(_) => {println!("fault"); break;}
                   }
                }
                Err(_) => {println!("Fault when reading data!"); break;}
            }
        }
        Err(e) => {println!("Connection failed with code {}", e); break;}
    }
}
}}