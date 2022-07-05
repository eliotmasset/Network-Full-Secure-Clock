
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
            stream.write(b"end").unwrap();

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
    answer_termios.c_lflag &= !(ICANON); // no echo and canonical mode
    tcsetattr(stdin, TCSANOW, &mut answer_termios).unwrap();
    let stdout = io::stdout();
    let mut reader = io::stdin();
    let mut buffer = [0;1];

    println!("            ---    The Network Full Secure Clock    ---");
    println!("            | Current Format : YYYY-MM-DD HH:mm:ss    |");
    println!("            | g : Get current time                    |");
    println!("            | s : Set the pattern timestamp           |");
    println!("            | t : Timestamp format                    |");
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
        't'=>showTimeStampTuto(),
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

fn showTimeStampTuto() {
    let stdin = 0;
    let termios = Termios::from_fd(stdin).unwrap();
    let mut answer_termios = termios.clone();
    answer_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
    tcsetattr(stdin, TCSANOW, &mut answer_termios).unwrap();
    let stdout = io::stdout();
    let mut reader = io::stdin();
    let mut buffer = [0;1];

    println!("---------------------------------   TimeStamp  Format   ---------------------------------");
    println!("| %Y (2001) The full proleptic Gregorian year, zero-padded to 4 digits.                 |");
    println!("| %C (20) The proleptic Gregorian year divided by 100, zero-padded to 2 digits.         |");
    println!("| %y (01) The proleptic Gregorian year modulo 100, zero-padded to 2 digits.             |");
    println!("| %m (07) Month number (01--12), zero-padded to 2 digits.                               |");
    println!("| %b (Jul) Abbreviated month name. Always 3 letters.                                    |");
    println!("| %B (July) Full month name. Also accepts corresponding abbreviation in parsing.        |");
    println!("| %h (Jul) Same as %b.                                                                  |");
    println!("| %d (08) Day number (01--31), zero-padded to 2 digits.                                 |");
    println!("| %e (8) Same as %d but space-padded. Same as %_d.                                      |");
    println!("| %a (Sun) Abbreviated weekday name. Always 3 letters.                                  |");
    println!("| %A (Sunday) Full weekday name. Also accepts corresponding abbreviation in parsing.    |");
    println!("| %w (0) Sunday = 0, Monday = 1, ..., Saturday = 6.                                     |");
    println!("| %u (7) Monday = 1, Tuesday = 2, ..., Sunday = 7. (ISO 8601)                           |");
    println!("| %U (28) Week number starting with Sunday (00--53), zero-padded to 2 digits.           |");
    println!("| %W (27) Same as %U, but week 1 starts with the first Monday in that year instead.     |");
    println!("| %G (2001) Same as %Y but uses the year number in ISO 8601 week date.                  |");
    println!("| %g (01) Same as %y but uses the year number in ISO 8601 week date.                    |");
    println!("| %V (27) Same as %U but uses the week number in ISO 8601 week date (01--53).           |");
    println!("| %j (189) Day of the year (001--366), zero-padded to 3 digits.                         |");
    println!("| %D (07/08/01) Month-day-year format. Same as %m/%d/%y.                                |");
    println!("| %x (07/08/01) Locale's date representation (e.g., 12/31/99).                          |");
    println!("| %F (2001-07-08) Year-month-day format (ISO 8601). Same as %Y-%m-%d.                   |");
    println!("| %v (8-Jul-2001) Day-month-year format. Same as %e-%b-%Y.                              |");
    println!("| --- TIME SPECIFIERS:                                                                  |");
    println!("| %H (00) Hour number (00--23), zero-padded to 2 digits.                                |");
    println!("| %k (0)     Same as %H but space-padded. Same as %_H.                                  |");
    println!("| %I (12) Hour number in 12-hour clocks (01--12), zero-padded to 2 digits.              |");
    println!("| %l (12) Same as %I but space-padded. Same as %_I.                                     |");
    println!("| %P (am) am or pm in 12-hour clocks.                                                   |");
    println!("| %p (AM) AM or PM in 12-hour clocks.                                                   |");
    println!("| %M (34) Minute number (00--59), zero-padded to 2 digits.                              |");
    println!("| %S (60) Second number (00--60), zero-padded to 2 digits.                              |");
    println!("| %f (026490000) The fractional seconds (in nanoseconds) since last whole second.       |");
    println!("| %.f (.026490) Similar to .%f but left-aligned. These all consume the leading dot.     |");
    println!("| %3f (026) Similar to %.3f but without the leading dot.                                |");
    println!("| %6f (026490) Similar to %.6f but without the leading dot.                             |");
    println!("| %9f (026490000) Similar to %.9f but without the leading dot.                          |");
    println!("| %R (00:34) Hour-minute format. Same as %H:%M.                                         |");
    println!("| %T (00:34:60) Hour-minute-second format. Same as %H:%M:%S.                            |");
    println!("| %X (00:34:60) Locale's time representation (e.g., 23:13:48).                          |");
    println!("| %r (12:34:60) AM Hour-minute-second format in 12-hour clocks. Same as %I:%M:%S %p.    |");
    println!("| --- TIME ZONE SPECIFIERS:                                                             |");
    println!("| %Z (ACST) Local time zone name. Skips all non-whitespace characters during parsing.   |");
    println!("| %z (+0930) Offset from the local time to UTC (with UTC being +0000).                  |");
    println!("| %:z (+09:30) Same as %z but with a colon.                                             |");
    println!("| %#z (+09) Parsing only: Same as %z but allows minutes to be missing or present.       |");
    println!("| --- DATE & TIME SPECIFIERS:                                                           |");
    println!("| %c (Sun Jul  8 00:34:60 2001) Locale's date and time (e.g., Thu Mar 3 23:05:25 2005). |");
    println!("| %+ (2001-07-08T00:34:60.026490+09:30) ISO 8601 / RFC 3339 date & time format.         |");
    println!("| %s (994518299) UNIX timestamp, the number of seconds since 1970-01-01 00:00 UTC.      |");
    println!("| --- SPECIAL SPECIFIERS:                                                               |");
    println!("| %t  Literal tab (\\t).                                                                 |");
    println!("| %n  Literal newline (\\n).                                                             |");
    println!("| %%  Literal percent sign.                                                             |");
    println!("-----------------------------------------------------------------------------------------");

    stdout.lock().flush().unwrap();
    reader.read_exact(&mut buffer).unwrap();
    let response = buffer[0] as char;
    tcsetattr(stdin, TCSANOW, & termios).unwrap();
    println!();
}