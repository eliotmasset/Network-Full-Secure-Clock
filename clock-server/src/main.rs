use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::str::from_utf8;
use chrono;
use chrono::prelude::*;
use regex::Regex;

fn getTime(pattern: &str) -> String {
    let now = Utc::now();
    let ts: i64 = now.timestamp();
    let nt = NaiveDateTime::from_timestamp(ts, 0);
    let dt: DateTime<Utc> = DateTime::from_utc(nt, Utc);
    let datetime = dt.format(pattern).to_string();
    return datetime;
}

fn handle_client(mut stream: TcpStream) {
    let mut datas = [0; 1024]; // using 50 byte buffer
    'reader: while match stream.read(&mut datas) {
        Ok(_) => {
            let data = datas;
            datas = [0; 1024];
            let resp = from_utf8(&data).unwrap().trim_matches(char::from(0));
            let re = Regex::new(r"^getTime:(.+)").unwrap();
            if re.is_match(resp) {
                let results = re.captures(resp).unwrap();
                let result = results.get(1).map_or("", |m| m.as_str());
                stream.write(getTime(result).as_bytes()).unwrap();
                stream.flush().unwrap();
            } else if resp=="end" {
                stream.flush().unwrap();
                stream.shutdown(Shutdown::Both).unwrap();
                break 'reader;
            } else {
                stream.write("invalid reponse".to_owned().as_bytes()).unwrap();
                stream.flush().unwrap();
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