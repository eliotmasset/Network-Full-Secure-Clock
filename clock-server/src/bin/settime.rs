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
use core::time::Duration;

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
        println!("Sorry, you need to start this program in root/admin");                  // ||
        return;                                                                           // ||
    }                                                                                     // \/ Set need capabilities

    let time : String =  format!("{:?}",env::args().nth_back(0));

    let id = ClockId::pid_cpu_clock_id(Pid::this()).ok().unwrap();
    println!("{}", if true { "y" } else { "n" });
    let r = clock_settime(id,TimeSpec::from_duration( Duration::from_millis(0)));
    println!("{}", if r.is_ok() { "y" } else { "n" });
    println!("{:?}", r.err().unwrap());
    println!("{:?}", r.ok().unwrap());
    println!("{}", if r.is_ok() { "y" } else { "n" });
    print!("yes");

    true;

}