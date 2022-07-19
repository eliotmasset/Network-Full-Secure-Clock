use caps::{Capability, CapSet};
use nix::unistd::Uid;
use nix::time::{clock_settime, ClockId};
use nix::sys::time::TimeSpec;
use std::env;
use core::time::Duration;
use std::str::FromStr;

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
    let timestamp : std::primitive::u64 =  std::primitive::u64::from_str(&env::args().nth_back(0).unwrap()).ok().unwrap();

    let r = clock_settime(ClockId::CLOCK_REALTIME,TimeSpec::from_duration(Duration::from_secs(timestamp)));
    if !r.is_ok() {
        panic!("{:?}", r.err().unwrap());
    }
    println!("true");

    true;

}