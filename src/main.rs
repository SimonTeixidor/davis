use mpd::Client;
use std::error::Error;
use std::net::TcpStream;

mod ansi;
mod error;
mod now_playing;
mod table;
mod tags;
mod terminal_dimensions;

fn main() -> Result<(), Box<dyn Error>> {
    let winsize = terminal_dimensions::terminal_size();
    let mut c = Client::new(TcpStream::connect("127.0.0.1:6600")?)?;
    now_playing::now_playing(&mut c, &winsize).unwrap();
    Ok(())
}
