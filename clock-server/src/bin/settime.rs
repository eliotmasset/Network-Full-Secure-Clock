use std::thread;
use std::net::{TcpListener, TcpStream};
use std::io::{Read};
use std::io::Write;
use caps::{Capability, CapSet};
use std::str::from_utf8;
use nix::unistd::Uid;
use nix::time::{clock_gettime, clock_settime, ClockId};
use nix::sys::time::TimeSpec;
use nix::unistd::Pid;
use std::env;
use chrono::Local;
use core::time::Duration;
use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};

fn main() {
    if !Uid::effective().is_root() {
        panic!("You must run this executable with root permissions");
    }
    caps::clear(None, CapSet::Effective).unwrap();                                        // ||
    caps::clear(None, CapSet::Inheritable).unwrap();                                      // ||
    let mut hset = caps::CapsHashSet::new();                                              // ||
    hset.insert(Capability::CAP_SYS_TIME);                                                // ||
    caps::set(None, CapSet::Effective, &hset).unwrap();                                   // ||
    caps::set(None, CapSet::Permitted, &hset).unwrap();                                   // ||
    for cap in caps::all() {                                                              // ||
        if cap != Capability::CAP_SYS_TIME {                                              // ||
            caps::drop(None,  CapSet::Permitted, cap).unwrap();                           // ||
        }                                                                                 // ||
    }                                                                                     // ||
    if !caps::has_cap(None, CapSet::Permitted, Capability::CAP_SYS_TIME).ok().unwrap() {  // ||
        panic!("Sorry, you need to start this program in root/admin");                    // ||
    }                                                                                     // \/ Set need capabilities

    let time_str : &str =  &env::args().nth_back(0).unwrap();
    let datetime = DateTime::parse_from_rfc3339(time_str);
    if !datetime.is_ok() {
        println!("{:?}", datetime.err());
        panic!("Wrong DateTime");
    }
    let timestamp = datetime.ok().unwrap().timestamp();

    let id_realtime = ClockId::CLOCK_REALTIME;
    
    let duration = Duration::from_secs(timestamp.try_into().unwrap());

    let r = clock_settime(id_realtime,TimeSpec::from_duration(duration));
    if !r.is_ok() {
        panic!("{:?}", r.err().unwrap());
    }
    println!("true");

    true;

}