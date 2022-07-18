use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::process::Command;
use chrono;
extern crate time;
use chrono::prelude::*;
use regex::Regex;
use std::time::SystemTime;
use caps::CapSet;

fn get_time(pattern: &str) -> String {
    let ts: i64 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).ok().unwrap().as_secs().try_into().unwrap();
    let nt = NaiveDateTime::from_timestamp(ts, 2);
    return Local.from_utc_datetime(&nt).format(pattern).to_string();
}

fn handle_client(mut stream: TcpStream) {
    let mut datas = [0; 1024];
    'reader: while match stream.read_exact(&mut datas) { // Wait exactly 1024 bytes
        Ok(_) => {
            let data = datas;
            datas = [0; 1024];
            let resp = from_utf8(&data).unwrap().trim_matches(char::from(0));
            let accept_chars="abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890+= |'\"²#%$€*!?.;,:/_-&éèëêîïüûàäâôöç@()[]{}ñ~°<>";
            let mut error = false;
            'chars : for resp_char in resp.chars() {
                let mut is_in = false;
                for accept_char in accept_chars.chars() {
                    if accept_char==resp_char {
                        is_in = true;
                    }
                }
                if !is_in {
                    error = true;
                    stream.write("invalid reponse".to_owned().as_bytes()).unwrap();
                    stream.flush().unwrap();
                    break 'chars;
                }
            }

            if !error {
                let re_get = Regex::new(r"^getTime:(.+)").unwrap();
                let re_set = Regex::new(r"^setTime:(.+)").unwrap();

                if re_get.is_match(resp) {
                    let results = re_get.captures(resp).unwrap();
                    let result = results.get(1).map_or("", |m| m.as_str());
                    stream.write(get_time(result).as_bytes()).unwrap();
                    stream.flush().unwrap();
                } else if re_set.is_match(resp) {
                    let results = re_set.captures(resp).unwrap();
                    let result = results.get(1).map_or("", |m| m.as_str());
                    let output = Command::new("sudo")
                                            .arg("./target/debug/settime")
                                            .output()
                                            .expect("failed to execute process");
                    let mut result="true";
                    println!("{}", from_utf8(&output.stdout).unwrap());
                    if from_utf8(&output.stdout).unwrap() != "yes" {
                        result="false";
                    }
                    stream.write(result.as_bytes()).unwrap();
                    stream.flush().unwrap();
                } else if resp=="end" {
                    stream.flush().unwrap();
                    stream.shutdown(Shutdown::Both).unwrap();
                    break 'reader;
                } else {
                    stream.write("invalid reponse".to_owned().as_bytes()).unwrap();
                    stream.flush().unwrap();
                }
            }
            true
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            false
        }
    } {}
}

fn main() {
    caps::clear(None, CapSet::Effective).unwrap();                                        // ||
    caps::clear(None, CapSet::Inheritable).unwrap();                                      // ||
    caps::clear(None, CapSet::Permitted).unwrap();                                        // \/ Clear capabilities

    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 8080");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move|| {
                    // connection succeeded
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
}