//Strobe systems
use std::net::{TcpListener};
use std::io::{Read, Write};
use std::time::{Duration, SystemTime};
use std::cmp::Ordering;
use std::{str, thread, fs::File};
use gpio_cdev::{Chip, LineRequestFlags};
use crossbeam_channel::unbounded;
use log::{warn, error, info, debug};
use simplelog;
fn main() {
    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(simplelog::LevelFilter::Info, simplelog::Config::default(), simplelog::TerminalMode::Mixed),
        simplelog::WriteLogger::new(simplelog::LevelFilter::Debug, simplelog::Config::default(), File::create("/var/log/EAS.log").unwrap()),
        ]).unwrap();
    let gpio_tupple: (gpio_cdev::LineHandle, gpio_cdev::LineHandle);
    match gpio_init(26, 20) {Ok(output) => {gpio_tupple = output;} Err(error)=>{panic!(error);}}
    let (genpin, silpin) = gpio_tupple;
    let mut general = Alarm {render_name: "General".to_string(), pin:genpin, active: false, activators: vec!(), start_time: SystemTime::UNIX_EPOCH};
    let mut silent = Alarm {render_name: "Silent".to_string(), pin:silpin, active: false, activators: vec!(), start_time: SystemTime::UNIX_EPOCH};
    let mut alarms = vec!(&mut general, &mut silent);
    let (threadcom_s, threadcom_r) = unbounded();
    info!("starting");
    let listener = TcpListener::bind("0.0.0.0:5400").unwrap();
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(mut streamm) => {
                    debug!("Got connection");
                    let mut data = [0 as u8; 50];
                    match streamm.read(&mut data){
                        Ok(size) => {
                        match str::from_utf8(&data[0..size]){
                            Ok(string_out) => {
                                let s: String = (&string_out).to_string();
                                info!("Got data {}", s);
                                let msg: Vec<u8>;
                                match threadcom_s.send(s){Ok(_)=> msg = b"ok".to_vec(), Err(e)=>{error!("{}",e); msg=b"fault".to_vec();}}
                                match streamm.write(&msg.as_slice()) {
                                Ok(_) => (),
                                Err(e) => {error!("Write Error: {}", e)}
                                }}
                            Err(_) => {error!("bad data"); break;}
                        }}
                        Err(_) => {error!("Fault when reading data!"); break;}
                    }}
                Err(e) => {error!("Connection failed with code {}", e);thread::sleep(Duration::from_secs(1));}
            }}
    });
    loop{
        thread::sleep(Duration::from_millis(100));
        match threadcom_r.try_recv(){
            Ok(out) => {
                let mut e = out.split(' ');
                match e.nth(0) {Some(alm)=>{
                for i in &mut alarms{
                    if alm.eq(&i.render_name){
                        &i.failsafe_reset();
                        match e.next(){Some(activator)=>{
                        if activator.eq("clear"){i.clear();}
                        else{i.add(activator.to_string());}
                        } None=>()}
                    }}}None=>{warn!("bad alarm data");}
                }}
            Err(_) => {} //usually will return an error as no data has been sent
        }
        for i in &mut alarms{i.update();}
    }
}
struct Alarm{
    render_name: String,
    pin: gpio_cdev::LineHandle,
    active: bool,
    activators: Vec<String>,
    start_time: std::time::SystemTime,
}
impl Alarm{
    fn update(&mut self) {
        self.active = !self.activators.is_empty();
        self.pin.set_value(!self.active as u8).unwrap(); //0 is on, 1 is off
        if !self.active{self.start_time = SystemTime::UNIX_EPOCH;}
        else{if self.start_time == SystemTime::UNIX_EPOCH{self.start_time = SystemTime::now();}
            else{
                match SystemTime::now().duration_since(self.start_time).unwrap().as_secs().cmp(&200) 
                {Ordering::Greater => self.clear(), _ => ()}
            }
        }
    }
    fn failsafe_reset(&mut self){//resets the failsafe timer every time a communication is recieved
        self.start_time = SystemTime::now();
    }
    fn clear(&mut self){self.activators.clear();}
    fn add(&mut self, act: String){
        if !self.activators.contains(&act) 
        {self.activators.push(act);}
    }
}
fn gpio_init(gen_pin: u32, sil_pin: u32) -> gpio_cdev::errors::Result<(gpio_cdev::LineHandle, gpio_cdev::LineHandle)>{
    let mut chip = Chip::new("/dev/gpiochip0")?;
    let genout_ln = chip.get_line(gen_pin)?;
    let silout_ln = chip.get_line(sil_pin)?;
    let genout = genout_ln.request(LineRequestFlags::OUTPUT, 1, "point-software")?;
    let silout = silout_ln.request(LineRequestFlags::OUTPUT, 1, "point-software")?;
    Ok((genout, silout))
}