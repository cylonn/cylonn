use std::old_io::BufferedReader;
use std::old_io::{Acceptor, Listener};
use std::old_io::net::pipe::{UnixListener, UnixStream};
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};

use uuid::Uuid;

// Handy class to generate messages.
struct Builder {
    client_id: u32,
}

impl Builder {

    fn new(client_id: u32) -> Self {
        Builder { client_id: client_id }
    }

    fn message(&self, event: Event) -> Message {
        Message { client_id: self.client_id, event: event }
    }

    fn line(&self, line: String) -> Message {
        self.message(Event::Line(line))
    }

    fn stream(&self, stream: UnixStream) -> Message {
        self.message(Event::Stream(stream))
    }

}

/// Messages are sent through channels. They are envelopes around Events.
pub struct Message {
    pub client_id: u32,
    pub event: Event,
}

/// Events are the "actual" payload of the Messages.
pub enum Event {
    Line(String),
    Stream(UnixStream),
}

// Handle a new socket.
fn handle(id: u32, sock: UnixStream, sender: Sender<Message>) {
    let builder = Builder::new(id);
    let mut reader = BufferedReader::new(sock.clone());
    // TODO: use Result of send() call
    sender.send(builder.stream(sock.clone()));

    // TODO: handle errors
    while let Ok(line) = reader.read_line() {
        // TODO: use Result of send() call
        sender.send(builder.line(line));
    }
}

// Accept sockets creating a thread for each new socket.
fn accept(path: String, sender: Sender<Message>) {
    let listener = UnixListener::bind(&path[..]).unwrap();
    let mut acceptor = listener.listen().unwrap();
    // Associate each client a unique id.
    let mut client_id = 0u32;

    while let Ok(sock) = acceptor.accept() {
        let s = sender.clone();
        thread::spawn(move || {
            handle(client_id, sock, s);
        });
        client_id += 1;
    }
}

/// Create a listener.
/// Returns a tuple with the socket's path and the receiving end of a channel.
pub fn create() -> (String, Receiver<Message>) {
    let path = format!("/tmp/{}.sock", Uuid::new_v4().to_simple_string());
    let (sender, receiver) = mpsc::channel::<Message>();

    let p = path.clone();
    thread::spawn(move || {
        accept(p, sender);
    });

    (path, receiver)
}
