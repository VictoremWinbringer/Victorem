extern crate victorem;

fn main() {
    let mut client = victorem::ClientSocket::new("1111", "127.0.0.1:2222").unwrap();
    loop {
        let _ = client
            .send(b"Ping!".to_vec())
            .map_err(|e| println!("{:#?}", e));
        let _ = client
            .recv()
            .map(|v| String::from_utf8(v).unwrap_or(String::new()))
            .map(|s| println!("From Server: {}", s));
    }
}
