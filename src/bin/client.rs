use std::{
    io::{
        self, ErrorKind, Read, Write
    },
    net::TcpStream,
    sync::mpsc::{
        self,
        TryRecvError
    },
    thread,
    time::Duration
};

const LOCAL: &str = "127.0.0.1:7878";
const MSG_SIZE: usize = 32;

// You can run several instances of the client binary and they will all connect to the server and be able to talk

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client.set_nonblocking(true).expect("failed to initiate non-blocking");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                println!("message recv {:?}", msg);
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with server was severed");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("writing to socket failed");
                println!("message sent {}", msg);
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                break;
            }
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("Write a message:");
    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("Failed to read message from stdin");
        let msg = buff.trim().to_string();
        if msg == ":quit" || tx.send(msg).is_err() {break};
    }

    println!("Bye!!");
}
