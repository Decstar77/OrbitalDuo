use orbital_shared::GamePacket;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

struct GameSession {
    player_a_stream: Option<TcpStream>,
    player_b_stream: Option<TcpStream>,
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:27007").unwrap();

    let game = Arc::new(Mutex::new(GameSession {
        player_a_stream: None,
        player_b_stream: None,
    }));

    for stream in listener.incoming() {
        println!("Connection established!");
        let stream = stream.unwrap();

        let game = Arc::clone(&game);
        thread::spawn(move || handle_connection(game, stream));
    }
}

fn handle_connection(game: Arc<Mutex<GameSession>>, stream: TcpStream) {
    let mut game = game.lock().unwrap();

    if game.player_a_stream.is_none() {
        game.player_a_stream = Some(stream.try_clone().unwrap());
    } else if game.player_b_stream.is_none() {
        game.player_b_stream = Some(stream.try_clone().unwrap());

        let player_a_stream = game.player_a_stream.as_ref().unwrap().try_clone().unwrap();
        let player_b_stream = game.player_b_stream.as_ref().unwrap().try_clone().unwrap();

        let (tx_a, rx_a) = std::sync::mpsc::channel::<Vec<u8>>();
        let (tx_b, rx_b) = std::sync::mpsc::channel::<Vec<u8>>();

        thread::spawn(move || relay_packets(player_a_stream, tx_b));
        thread::spawn(move || relay_packets(player_b_stream, tx_a));

        //thread::spawn(move || send_packets(game.player_a_stream.as_mut().unwrap(), rx_a));
        // thread::spawn(move || send_packets(game.player_b_stream.as_mut().unwrap(), rx_b));
    }
}

fn relay_packets(mut stream: TcpStream, tx: std::sync::mpsc::Sender<Vec<u8>>) {
    loop {
        let mut buffer = [0; 1024];
        let read_result = stream.read(&mut buffer);
        match read_result {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    let packet = buffer[..bytes_read].to_vec();
                    tx.send(packet).unwrap();
                } else {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

fn send_packets(stream: &mut TcpStream, rx: std::sync::mpsc::Receiver<Vec<u8>>) {
    loop {
        match rx.recv() {
            Ok(packet) => {
                if let Err(_) = stream.write_all(&packet) {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}
