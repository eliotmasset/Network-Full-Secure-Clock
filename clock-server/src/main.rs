use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::process::Command;
use chrono;
extern crate time;
use std::io;
use chrono::prelude::*;
use regex::Regex;
use std::time::SystemTime;
use caps::CapSet;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use ansi_term::Colour;
use crossterm::{cursor};
use sha256::digest_file;
use std::path::Path;

const sha256_settime : &str ="9bc642372795d5308c2d5ee1949ce56b9ab778c83365e8b398458043ef2f826b";


fn set_time() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    let style_green_bold = Colour::Green.bold();
    let style_yellow_bold = Colour::Yellow.bold();
    println!("{}{}{}",style_green_bold.paint("Please, enter the new date ("),style_yellow_bold.paint("format rfc3339 YYYY-MM-DDTHH:MM:SS+00:00"),style_green_bold.paint("):"));

    let accept_chars="TZ1234567890%:+- ";

    let mut rl = Editor::<()>::new();
    let readline = rl.readline(">> ");
    let style_red_bold = Colour::Red.bold();
    match readline {
        Ok(line) => {
            for resp_char in line.chars() {
                let mut is_in = false;
                for accept_char in accept_chars.chars() {
                    if accept_char==resp_char {
                        is_in = true;
                    }
                }
                if !is_in {
                    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                    println!("{}{}", style_red_bold.paint("UNOTHORIZED char : "),style_yellow_bold.paint(String::from(resp_char)));
                    return;
                }
            }
            let len : i16 = line.len().try_into().unwrap();
            if len > 100 {
                print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                println!("{}", style_red_bold.paint("DATE TOO LONG, sorry, the date must be less than 100 chars"));
            } else {
                let style_red_bold = Colour::Red.bold();
                let style_green_bold = Colour::Green.bold();
                let input = Path::new("./target/debug/settime");
                let res = digest_file(input).unwrap();
                print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                if res != sha256_settime {
                    panic!("CRITICAL : WRONG SHA256 FOR APP SETTIME");
                }
                let output = Command::new("sudo")
                                        .arg("./target/debug/settime")
                                        .arg(line)
                                        .output()
                                        .expect("failed to execute process");
                let mut response="true";
                if from_utf8(&output.stdout).unwrap().trim_matches('\n') != "true" {
                    response="false";
                }

                if response == "true" {
                    println!("{}", style_green_bold.paint("Time set SuccesFull!"));
                } else {
                    println!("{}",style_red_bold.paint("Time set Error!"));
                }
            }
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
}

fn get_time(pattern: &str) -> String {
    let ts: i64 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).ok().unwrap().as_secs().try_into().unwrap();
    let nt = NaiveDateTime::from_timestamp(ts, 0);
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

                if re_get.is_match(resp) {
                    let results = re_get.captures(resp).unwrap();
                    let result = results.get(1).map_or("", |m| m.as_str());
                    stream.write(get_time(result).as_bytes()).unwrap();
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

fn menu(mut timestamp_pattern: &mut String) -> i32 {
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
    }else if y==1 {
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
            style_yellow_bold.paint("d"),
            style_red_bold.paint(" : Set a new date                      |"));
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
    
    return ask(response, &mut timestamp_pattern);
}

fn ask(c : char, mut timestamp_pattern:  &mut String) -> i32 {
    if c=='e' {
        return 0;
    }

    match c {
        'e'=>return 0,
        's'=>set_pattern_str(&mut timestamp_pattern),
        'g'=>get_time_str(&timestamp_pattern),
        'd'=>set_time(),
        't'=>show_time_stamp_tuto(),
        _=>{
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            println!("Invalid response!");
        },
    }

    println!();
    return 1;
}

fn main() {
    caps::clear(None, CapSet::Effective).unwrap();                                        // ||
    caps::clear(None, CapSet::Inheritable).unwrap();                                      // ||
    caps::clear(None, CapSet::Permitted).unwrap();                                        // \/ Clear capabilities

    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 8080");
    thread::spawn(move|| {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move|| {
                        // connection succeeded
                        handle_client(stream);
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
    });

    let mut timestamp_pattern = "%Y-%m-%d %H:%M:%S".to_string();
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

    loop {
        let ret = menu(&mut timestamp_pattern);
        if ret == 0 {
            break;
        }
    }
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

fn get_time_str(timestamp_pattern: &String) {
    let style_red_bold = Colour::Red.bold();
    let style_green_bold = Colour::Green.bold();
    let text = get_time(timestamp_pattern);
    if text != "invalid reponse" {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        println!("{}{}{}",
                style_red_bold.paint("Current Time : [ "),
                style_green_bold.paint(text),
                style_red_bold.paint(" ]"));
    }   else {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        println!("{}",style_red_bold.paint("          Error : No current time available"));
    }
}

fn set_pattern_str(timestamp_pattern: &mut String) {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    let style_green_bold = Colour::Green.bold();
    println!("{}",style_green_bold.paint("Please, enter the new pattern :"));

    let mut rl = Editor::<()>::new();
    let readline = rl.readline(">> ");
    let mut resp = String::from("");
    let style_red_bold = Colour::Red.bold();
    let style_yellow_bold = Colour::Yellow.bold();
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