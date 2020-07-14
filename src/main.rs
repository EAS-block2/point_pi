//Digital Signage and Strobe systems
use std::net::{TcpListener};
use std::io::{Read, Write};
use std::time::Duration;
use std::{str, thread};
use gpio::GpioOut;
use crossbeam_channel::unbounded;
fn main() {
    let mut general = Alarm {render_name: "General".to_string(), pin:25, active: false, activators: vec!()};
    let mut silent = Alarm {render_name: "Silent".to_string(), pin:28, active: false, activators: vec!()};
    let mut alarms = vec!(&mut general, &mut silent);
    let (threadcom_s, threadcom_r) = unbounded();
    println!("starting");
    let listener = TcpListener::bind("0.0.0.0:5400").unwrap();
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
                                match streamm.write(&msg.as_slice()) {
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
        println!("main loop run");
        thread::sleep(Duration::from_millis(500));
        match threadcom_r.try_recv(){
            Ok(out) => {
                let mut e = out.split(' ');
                match e.nth(0) {Some(alm)=>{
                for i in &mut alarms{
                    if alm.eq(&i.render_name){
                        match e.next(){Some(activator)=>{
                        if activator.eq("clear"){i.clear();}
                        else{i.add(activator.to_string());}
                        } None=>()}
                    }}}None=>{println!("bad alarm data");}
                }}
            Err(_) => {thread::sleep(Duration::from_secs(2));
            println!("no new data, sleeping");} //usually will return an error as no data has been sent
        }
        for i in &mut alarms{
            i.update();
            println!("{} alarm is {}, activated by {:?}", i.render_name, i.active, i.activators);
        }
    }
}
struct Alarm{
    render_name: String,
    pin: u16,
    active: bool,
    activators: Vec<String>
}
impl Alarm{
    fn update(&mut self){self.active = !self.activators.is_empty();
    gpio::sysfs::SysFsGpioOutput::open(self.pin).unwrap().set_value(self.active).unwrap();}
    fn clear(&mut self){self.activators.clear();}
    fn add(&mut self, act: String){
        if !self.activators.contains(&act) 
        {self.activators.push(act);}
    }
}
