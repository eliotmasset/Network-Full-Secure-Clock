
//Dépendencies 
extern crate termios;
use std::io;
use std::io::Read;
use std::io::Write;
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};


// Main program function
fn main() {
    loop {
        let ret = menu();
        if ret == 0 {
            break;
        }
    }    
}

fn menu() -> i32 {
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
    println!("            | s : Set the timestamp                   |");
    println!("            | e : Exit                                |");
    println!("            -------------------------------------------");

    let mut wait : char;

    stdout.lock().flush().unwrap();
    reader.read_exact(&mut buffer).unwrap();
    let response = buffer[0] as char;
    tcsetattr(stdin, TCSANOW, & termios).unwrap();
    println!();
    return ask(response);
}

fn ask(c : char) -> i32 {
    if c=='e' {
        return 0;
    }
    match c {
        'e'=>return 0,
        _=>println!("Invalid response!"),
    }
    println!();
    return 1;
}