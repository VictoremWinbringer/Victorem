extern crate victorem;

use rand;
use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let p: u16 = 1111;// rng.gen_range(1111, 60000);
    let mut client = victorem::ClientSocket::new(&format!("{}", p), "127.0.0.1:2222").unwrap();
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
