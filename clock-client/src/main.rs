
//Dépendencies 
extern crate termios;
extern crate rustyline;
extern crate crossterm;
use std::io;
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};
use std::net::{TcpStream, Shutdown};
use std::io::{Read, Write};
use std::str::from_utf8;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use ansi_term::Colour;
use crossterm::{cursor};
use std::any::type_name;

// Main program function
fn main() {
    match TcpStream::connect("localhost:8080") {
        Ok(mut stream) => {
            println!("Successfully connected to server");
            let mut timestamp_pattern = "%Y-%m-%d %H:%M:%S".to_string();
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

            loop {
                let ret = menu(&stream, &mut timestamp_pattern);
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
    let style_yellow_bold = Colour::Yellow.bold();
    println!("{}",style_yellow_bold.paint("GoodBye!"));
    println!("");
}

fn menu(stream: &TcpStream, mut timestamp_pattern: &mut String) -> i32 {
    let stdin = 0;
    let termios = Termios::from_fd(stdin).unwrap();
    let mut answer_termios = termios.clone();
    answer_termios.c_lflag &= !(ICANON); // no echo and canonical mode
    tcsetattr(stdin, TCSANOW, &mut answer_termios).unwrap();
    let stdout = io::stdout();
    let mut reader = io::stdin();
    let mut buffer = [0;1];

    let len : i16 = timestamp_pattern.len().try_into().unwrap();
    let nb_spaces : i16 = 23-len;
    let mut timestamp_pattern_spaces = timestamp_pattern.clone();
    if nb_spaces > 0 {
        for _ in  0..nb_spaces {
            timestamp_pattern_spaces.push(' ');
        }
    }

    let title = "The Network Full Secure Clock";
    let style_red_bold = Colour::Red.bold();
    let style_green_bold = Colour::Green.bold();
    let style_yellow_bold = Colour::Yellow.bold();

    let (_,y) =  cursor::position().ok().unwrap();
    if y==0 {
        println!("");
        println!("");
    }

    println!("{}{}{}", 
            style_red_bold.paint("            ---    "),
            style_red_bold.paint(title),
            style_red_bold.paint("    ---"));
    println!("{}{}{}",
            style_red_bold.paint("            | Current Format : "),
            style_green_bold.paint(timestamp_pattern_spaces),
            style_red_bold.paint("|"));
    println!("{}{}{}",
            style_red_bold.paint("            | "),
            style_yellow_bold.paint("g"),
            style_red_bold.paint(" : Get current time                    |"));
    println!("{}{}{}",
            style_red_bold.paint("            | "),
            style_yellow_bold.paint("s"),
            style_red_bold.paint(" : Set the pattern timestamp           |"));
    println!("{}{}{}",
            style_red_bold.paint("            | "),
            style_yellow_bold.paint("t"),
            style_red_bold.paint(" : Timestamp format                    |"));
    println!("{}{}{}",
            style_red_bold.paint("            | "),
            style_yellow_bold.paint("e"),
            style_red_bold.paint(" : Exit                                |"));
    println!("{}",
            style_red_bold.paint("            -------------------------------------------"));

    stdout.lock().flush().unwrap();
    reader.read_exact(&mut buffer).unwrap();
    let response = buffer[0] as char;
    tcsetattr(stdin, TCSANOW, & termios).unwrap();
    println!("\r ");
    
    return ask(response, &stream, &mut timestamp_pattern);
}

fn ask(c : char, stream: &TcpStream, mut timestamp_pattern:  &mut String) -> i32 {
    if c=='e' {
        return 0;
    }

    match c {
        'e'=>return 0,
        's'=>set_pattern(&mut timestamp_pattern),
        'g'=>get_time(&stream, &timestamp_pattern),
        't'=>show_time_stamp_tuto(),
        _=>{
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            println!("Invalid response!");
        },
    }

    println!();
    return 1;
}

fn get_time(mut stream: &TcpStream, timestamp_pattern: &String) {
    let msg = format!("{}{}","getTime:",timestamp_pattern);
    let msg_bytes = msg.as_bytes();
    stream.write(msg_bytes).unwrap();

    let mut data = [0; 1024]; // using 6 byte buffer
    match stream.read(&mut data) {
        Ok(_) => {
            let style_red_bold = Colour::Red.bold();
            let style_green_bold = Colour::Green.bold();
            let text = from_utf8(&data).unwrap().trim_matches(char::from(0));
            if text != "invalid reponse" {
                print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                println!("{}{}{}",
                        style_red_bold.paint("Current Time : [ "),
                        style_green_bold.paint(text),
                        style_red_bold.paint(" ]"));
            }   else {
                println!("          Error : No current time available");
            }
            stream.flush().unwrap();
        },
        Err(e) => {
            println!("Failed to receive data: {}", e);
        }
    }
}

fn set_pattern(timestamp_pattern: &mut String) {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    let style_green_bold = Colour::Green.bold();
    println!("{}",style_green_bold.paint("Please, enter the new pattern :"));

    let mut rl = Editor::<()>::new();
    let readline = rl.readline(">> ");
    let mut resp = String::from("");
    match readline {
        Ok(line) => {
            resp = line;
        },
        Err(ReadlineError::Interrupted) => {
            println!("CTRL-C");
        },
        Err(ReadlineError::Eof) => {
            println!("CTRL-D");
        },
        Err(err) => {
            println!("Error: {:?}", err);
        }
    }
    *timestamp_pattern = format!("{}",resp.trim_matches(char::from(0)).trim_matches('\n').trim_matches(char::from(10)).to_string());
    
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn show_time_stamp_tuto() {
    let style_red_bold = Colour::Red.bold();
    let style_yellow_bold = Colour::Yellow.bold();
    let style_blue_bold = Colour::Blue.bold();
    let stdin = 0;
    let termios = Termios::from_fd(stdin).unwrap();
    let mut answer_termios = termios.clone();
    answer_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
    tcsetattr(stdin, TCSANOW, &mut answer_termios).unwrap();
    let stdout = io::stdout();
    let mut reader = io::stdin();
    let mut buffer = [0;1];

    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("{}{}{}", 
                style_red_bold.paint("---------------------------------   "),
                style_blue_bold.paint("TimeStamp  Format"),
                style_red_bold.paint("   ---------------------------------"));
    println!("{}{}{}", 
                style_red_bold.paint( "| "),
                style_yellow_bold.paint("%Y"),
                style_red_bold.paint(" (2001) The full proleptic Gregorian year, zero-padded to 4 digits.                 |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%C"),
                style_red_bold.paint(" (20) The proleptic Gregorian year divided by 100, zero-padded to 2 digits.         |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%y"),
                style_red_bold.paint(" (01) The proleptic Gregorian year modulo 100, zero-padded to 2 digits.             |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%m"),
                style_red_bold.paint(" (07) Month number (01--12), zero-padded to 2 digits.                               |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%b"),
                style_red_bold.paint(" (Jul) Abbreviated month name. Always 3 letters.                                    |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%B"),
                style_red_bold.paint(" (July) Full month name. Also accepts corresponding abbreviation in parsing.        |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%h"),
                style_red_bold.paint(" (Jul) Same as %b.                                                                  |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%d"),
                style_red_bold.paint(" (08) Day number (01--31), zero-padded to 2 digits.                                 |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%e"),
                style_red_bold.paint(" (8) Same as %d but space-padded. Same as %_d.                                      |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%a"),
                style_red_bold.paint(" (Sun) Abbreviated weekday name. Always 3 letters.                                  |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%A"),
                style_red_bold.paint(" (Sunday) Full weekday name. Also accepts corresponding abbreviation in parsing.    |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%w"),
                style_red_bold.paint(" (0) Sunday = 0, Monday = 1, ..., Saturday = 6.                                     |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%u"),
                style_red_bold.paint(" (7) Monday = 1, Tuesday = 2, ..., Sunday = 7. (ISO 8601)                           |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%U"),
                style_red_bold.paint(" (28) Week number starting with Sunday (00--53), zero-padded to 2 digits.           |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%W"),
                style_red_bold.paint(" (27) Same as %U, but week 1 starts with the first Monday in that year instead.     |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%G"),
                style_red_bold.paint(" (2001) Same as %Y but uses the year number in ISO 8601 week date.                  |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%g"),
                style_red_bold.paint(" (01) Same as %y but uses the year number in ISO 8601 week date.                    |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%V"),
                style_red_bold.paint(" (27) Same as %U but uses the week number in ISO 8601 week date (01--53).           |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%j"),
                style_red_bold.paint(" (189) Day of the year (001--366), zero-padded to 3 digits.                         |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%D"),
                style_red_bold.paint(" (07/08/01) Month-day-year format. Same as %m/%d/%y.                                |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%x"),
                style_red_bold.paint(" (07/08/01) Locale's date representation (e.g., 12/31/99).                          |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%F"),
                style_red_bold.paint(" (2001-07-08) Year-month-day format (ISO 8601). Same as %Y-%m-%d.                   |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%v"),
                style_red_bold.paint(" (8-Jul-2001) Day-month-year format. Same as %e-%b-%Y.                              |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "),
                style_blue_bold.paint("--- TIME SPECIFIERS:                                                                  "),
                style_red_bold.paint("|"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%H"),
                style_red_bold.paint(" (00) Hour number (00--23), zero-padded to 2 digits.                                |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%k"),
                style_red_bold.paint(" (0)     Same as %H but space-padded. Same as %_H.                                  |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%I"),
                style_red_bold.paint(" (12) Hour number in 12-hour clocks (01--12), zero-padded to 2 digits.              |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%l"),
                style_red_bold.paint(" (12) Same as %I but space-padded. Same as %_I.                                     |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%P"),
                style_red_bold.paint(" (am) am or pm in 12-hour clocks.                                                   |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%p"),
                style_red_bold.paint(" (AM) AM or PM in 12-hour clocks.                                                   |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%M"),
                style_red_bold.paint(" (34) Minute number (00--59), zero-padded to 2 digits.                              |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%S"),
                style_red_bold.paint(" (60) Second number (00--60), zero-padded to 2 digits.                              |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%f"),
                style_red_bold.paint(" (026490000) The fractional seconds (in nanoseconds) since last whole second.       |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%.f"),
                style_red_bold.paint(" (.026490) Similar to .%f but left-aligned. These all consume the leading dot.     |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%3f"),
                style_red_bold.paint(" (026) Similar to %.3f but without the leading dot.                                |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%6f"),
                style_red_bold.paint(" (026490) Similar to %.6f but without the leading dot.                             |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%9f"),
                style_red_bold.paint(" (026490000) Similar to %.9f but without the leading dot.                          |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%R"),
                style_red_bold.paint(" (00:34) Hour-minute format. Same as %H:%M.                                         |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%T"),
                style_red_bold.paint(" (00:34:60) Hour-minute-second format. Same as %H:%M:%S.                            |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%X"),
                style_red_bold.paint(" (00:34:60) Locale's time representation (e.g., 23:13:48).                          |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%r"),
                style_red_bold.paint(" (12:34:60) AM Hour-minute-second format in 12-hour clocks. Same as %I:%M:%S %p.    |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "),
                style_blue_bold.paint("--- TIME ZONE SPECIFIERS:                                                             "),
                style_red_bold.paint("|"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%Z"),
                style_red_bold.paint(" (ACST) Local time zone name. Skips all non-whitespace characters during parsing.   |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%z"),
                style_red_bold.paint(" (+0930) Offset from the local time to UTC (with UTC being +0000).                  |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%:z"),
                style_red_bold.paint(" (+09:30) Same as %z but with a colon.                                             |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%#z"),
                style_red_bold.paint(" (+09) Parsing only: Same as %z but allows minutes to be missing or present.       |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "),
                style_blue_bold.paint("--- DATE & TIME SPECIFIERS:                                                           "),
                style_red_bold.paint("|"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%c"),
                style_red_bold.paint(" (Sun Jul  8 00:34:60 2001) Locale's date and time (e.g., Thu Mar 3 23:05:25 2005). |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%+"),
                style_red_bold.paint(" (2001-07-08T00:34:60.026490+09:30) ISO 8601 / RFC 3339 date & time format.         |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%s"),
                style_red_bold.paint(" (994518299) UNIX timestamp, the number of seconds since 1970-01-01 00:00 UTC.      |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "),
                style_blue_bold.paint("--- SPECIAL SPECIFIERS:                                                               "),
                style_red_bold.paint("|"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%t"),
                style_red_bold.paint("  Literal tab (\\t).                                                                 |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%n"),
                style_red_bold.paint("  Literal newline (\\n).                                                             |"));
    println!("{}{}{}", 
                style_red_bold.paint("| "), 
                style_yellow_bold.paint("%%"),
                style_red_bold.paint("  Literal percent sign.                                                             |"));
    println!("{}",style_red_bold.paint("-----------------------------------------------------------------------------------------"));
    println!("{}",style_yellow_bold.paint(" Press any keys to exit"));

    stdout.lock().flush().unwrap();
    reader.read_exact(&mut buffer).unwrap();
    tcsetattr(stdin, TCSANOW, & termios).unwrap();
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}