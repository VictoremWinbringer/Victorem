mod entities;
mod data_access_layer;

trait Game {
    fn update(&mut self, delta_time:std::time::Duration, commands:Vec<Vec<u8>>, from_address: &str) -> Vec<u8>;
}

struct GameProxy {
    game: std::sync::Arc<std::sync::Mutex<Game>>
}

impl GameProxy {
    fn new(game: std::sync::Arc<std::sync::Mutex<Game>>) -> GameProxy {
        GameProxy{game}
    }

    fn update(&mut self, delta_time:std::time::Duration, commands:Vec<Vec<u8>>, from_address: &str) -> Vec<u8>{
        let mut game = self.game.lock().unwrap();
        game.update(delta_time,commands,from_address)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    #[test]
    fn it_works() {
        assert_eq!(1,1);
    }
}
