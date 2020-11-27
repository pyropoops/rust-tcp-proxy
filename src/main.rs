use std::io::Read;
use std::io::Write;
use std::{
    io::Error,
    net::{TcpListener, TcpStream},
    thread,
};

const BIND: &str = "0.0.0.0:1337";
const REDIRECT: &str = "127.0.0.1:3389";
const BUFFER_SIZE: usize = 1024;

fn main() {
    match start() {
        Ok(_) => (),
        Err(err) => println!("{}", err),
    }
}

fn start() -> Result<(), Error> {
    let listener = TcpListener::bind(BIND)?;
    for incoming in listener.incoming() {
        thread::spawn(move || handle_conn(incoming?));
    }
    Ok(())
}

fn handle_conn(stream: TcpStream) -> Result<(), Error> {
    println!("Incoming connection...");
    let redirect = TcpStream::connect(REDIRECT)?;
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
