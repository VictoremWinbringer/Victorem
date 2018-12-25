mod entities;
mod data_access_layer;

trait Game {
    fn update(&mut self, delta_time: std::time::Duration, commands: Vec<Vec<u8>>, from_address: &str) -> Vec<u8>;
}

struct GameProxy {
    game: std::sync::Arc<std::sync::Mutex<Game>>
}

impl GameProxy {
    fn new(game: std::sync::Arc<std::sync::Mutex<Game>>) -> GameProxy {
        let mut client = crate::data_access_layer::TypedClientSocket::new("sdsf", "sdfsf").unwrap();
        let mut server = crate::data_access_layer::TypedServerSocket::new("asdfaf").unwrap();
        GameProxy { game }
    }

    fn update(&mut self, delta_time: std::time::Duration, commands: Vec<Vec<u8>>, from_address: &str) -> Vec<u8> {
        let mut game = self.game.lock().unwrap();
        game.update(delta_time, commands, from_address)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::thread;
    use threadpool::ThreadPool;

    #[test]
    fn it_works() {
        let pool = ThreadPool::new(100);
        for i in 0..1000 {
            pool.execute(move || {
                thread::sleep_ms(100);
                println!("value {} from thread {:?}", i, thread::current().id());
            })
        }

        thread::sleep_ms(1100);
        assert_eq!(1, 1);
    }
}
