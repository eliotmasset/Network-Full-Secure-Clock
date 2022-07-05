
//Dépendencies 
extern crate termios;
use std::io;
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};
use std::net::{TcpStream, Shutdown};
use std::io::{Read, Write};
use std::str::from_utf8;


// Main program function
fn main() {
    match TcpStream::connect("localhost:8080") {
        Ok(mut stream) => {
            println!("Successfully connected to server");

            loop {
                let ret = menu(&stream);
                if ret == 0 {
                    break;
                }
            }
            stream.flush().unwrap();
            stream.write(b"end").unwrap();
            stream.flush().unwrap();
            stream.shutdown(Shutdown::Both).unwrap();

        },
        Err(e) => {
            println!("Failed to connect to the clock server: {}", e);
        }
    }  
    println!("GoodBy!");
}

fn menu(mut stream: &TcpStream) -> i32 {
    let stdin = 0;
    let termios = Termios::from_fd(stdin).unwrap();
    let mut answer_termios = termios.clone();
    answer_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
    tcsetattr(stdin, TCSANOW, &mut answer_termios).unwrap();
    let stdout = io::stdout();
    let mut reader = io::stdin();
    let mut buffer = [0;1];

    println!("            ---    The Network Full Secure Clock    ---");
    println!("            | Current Format : YYYY-MM-DD HH:mm:ss    |");
    println!("            | g : Get current time                    |");
    println!("            | s : Set the pattern timestamp           |");
    println!("            | e : Exit                                |");
    println!("            -------------------------------------------");

    stdout.lock().flush().unwrap();
    reader.read_exact(&mut buffer).unwrap();
    let response = buffer[0] as char;
    tcsetattr(stdin, TCSANOW, & termios).unwrap();
    println!();
    return ask(response, &stream);
}

fn ask(c : char, mut stream: &TcpStream) -> i32 {
    if c=='e' {
        return 0;
    }

    match c {
        'e'=>return 0,
        'g'=>getTime(&stream),
        _=>println!("Invalid response!"),
    }

    println!();
    return 1;
}

fn getTime(mut stream: &TcpStream) {
    let msg = b"getTime";

    stream.write(msg).unwrap();

    let mut data = [0; 1024]; // using 6 byte buffer
    match stream.read(&mut data) {
        Ok(_) => {
            let text = from_utf8(&data).unwrap();
            println!("          Current Time : [ {} ]", text);
            data = [0; 1024];
            stream.flush().unwrap();
        },
        Err(e) => {
            println!("Failed to receive data: {}", e);
        }
    }
}