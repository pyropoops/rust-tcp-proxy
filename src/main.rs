use std::io::Read;
use std::io::Write;
use std::{
    io::Error,
    net::{TcpListener, TcpStream},
    thread,
};

const BUFFER_SIZE: usize = 1024;

fn main() {
    let mut args = std::env::args();
    if args.len() < 3 {
        let cmd = args.nth(0);
        if cmd.is_some() {
            println!("Usage: {} <bind_addr> <redirect_addr>", cmd.unwrap());
        } else {
            println!("Usage: proxy <bind_addr> <redirect_addr>");
        }
        return;
    }
    match start(args.nth(1).unwrap(), args.nth(2).unwrap()) {
        Ok(_) => (),
        Err(err) => println!("{}", err),
    }
}

fn start(bind: String, redirect: String) -> Result<(), Error> {
    let listener = TcpListener::bind(bind)?;
    for incoming in listener.incoming() {
        let redirect = TcpStream::connect(&redirect);
        thread::spawn(move || handle_conn(incoming?, redirect?));
    }
    Ok(())
}

fn handle_conn(stream: TcpStream, redirect: TcpStream) -> Result<(), Error> {
    println!("Incoming connection...");
    let connections = vec![
        pipe_stream(stream.try_clone()?, redirect.try_clone()?),
        pipe_stream(redirect.try_clone()?, stream.try_clone()?),
    ];
    for connection in connections {
        match connection.join() {
            Ok(_) => (),
            Err(_) => println!("There was an internal error joining threads..."),
        }
    }
    Ok(())
}

fn pipe_stream(mut read: TcpStream, mut write: TcpStream) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        let mut buf = [0; BUFFER_SIZE];
        match read.read(&mut buf) {
            Ok(len) => {
                if len == 0 {
                    break;
                }
                match write.write_all(&buf[..len]) {
                    Ok(_) => (),
                    Err(_) => break,
                };
            }
            Err(_) => break,
        };
    })
}
