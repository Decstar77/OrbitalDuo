use std::net::{TcpStream};
use std::io::{self, Error};

pub struct NetworkState {
    stream: Option<TcpStream>,
}

impl NetworkState {
    pub fn new() -> NetworkState {
        NetworkState { stream: None }
    }

    pub fn connect_to(&mut self, ip_port: &str) -> io::Result<()> {
        let stream = TcpStream::connect(ip_port)?;
        self.stream = Some(stream);
        Ok(())
    }
}


mod tests
{
    use super::*;

    #[test]
    fn test_connect_to() {
        let mut network_state = NetworkState::new();
        match network_state.connect_to("127.0.0.1:27007") {
            Ok(_) => println!("Connected to the server successfully!"),
            Err(e) => println!("Failed to connect to the server: {}", e),
        }
    }
}