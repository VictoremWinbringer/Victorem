use std::borrow::Borrow;
use std::error::Error;
use std::net::SocketAddr;
use std::ops::{Add, Mul};
use std::time::Duration;
use victorem;
use victorem::{ClientSocket, ContinueRunning, Exception, Game, GameServer, ServerEvent};

struct GameData {
    events: Vec<ServerEvent>,
    updates: Vec<(Duration, Vec<Vec<u8>>, SocketAddr)>,
    continue_on_command: bool,
    disconnect_this_client: Option<SocketAddr>,
    draw: Vec<u8>,
    drawn: Vec<Duration>,
    new_client: Option<SocketAddr>,
    continue_on_event: bool,
}

impl GameData {
    fn new() -> GameData {
        GameData {
            events: Vec::new(),
            updates: Vec::new(),
            continue_on_command: true,
            disconnect_this_client: None,
            draw: Vec::new(),
            drawn: Vec::new(),
            new_client: Some(SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                7777,
            )),
            continue_on_event: true,
        }
    }
}

struct GameMock<'a> {
    data: &'a mut GameData,
    counter: usize,
    current: usize,
}

impl<'a> GameMock<'a> {
    fn new(data: &'a mut GameData, counter: usize) -> GameMock {
        GameMock {
            data,
            counter,
            current: 0,
        }
    }
}

impl<'a> Game for GameMock<'a> {
    fn handle_command(
        &mut self,
        delta_time: Duration,
        commands: Vec<Vec<u8>>,
        from: SocketAddr,
    ) -> bool {
        self.data.updates.push((delta_time, commands, from));
        self.data.continue_on_command
    }

    fn draw(&mut self, delta_time: Duration) -> Vec<u8> {
        self.data.drawn.push(delta_time);
        self.current += 1;
        if self.current > self.counter {
            self.data.continue_on_event = false;
        }
        self.data.draw.clone()
    }

    fn handle_server_event(&mut self, event: ServerEvent) -> ContinueRunning {
        self.data.events.push(event);
        self.data.continue_on_event.clone()
    }
    fn add_client(&mut self) -> Option<SocketAddr> {
        self.data.new_client.clone()
    }
}

fn create_server(game: GameMock, port: String) -> Result<GameServer<GameMock>, Exception> {
    GameServer::new(game, &port)
}

#[test]
fn server_should_send_state_to_client_on_draw() -> Result<(), Exception> {
    std::thread::spawn(|| {
        let mut game_data = GameData::new();
        game_data.draw = vec![3u8, 7, 8];
        let game_mock = GameMock::new(&mut game_data, 100000);
        if let Ok(mut game_server) = create_server(game_mock, "3336".into()) {
            game_server.run();
        }
    });
    let mut client = ClientSocket::new("4444", "127.0.0.1:3336")?;
    client.send(vec![1u8]);
    client.send(vec![1u8]);
    client.send(vec![1u8]);
    let res = loop {
        match client.recv() {
            Ok(r) => break r,
            Err(_) => continue,
        }
    };
    assert_eq!(vec![3u8, 7, 8], res);
    Ok(())
}

#[test]
fn server_should_stop_if_handle_command_returns_false() -> Result<(), Exception> {
    std::thread::spawn(|| {
        let res = ClientSocket::new("1112", "127.0.0.1:3333")
            .map(|mut c| {
                for _i in 0..1000 {
                    c.send(vec![1u8, 3u8]);
                }
                1
            })
            .unwrap_or(0);
        res
    });
    let timer = std::time::Instant::now();
    let start = timer.elapsed();
    let mut game_data = GameData::new();
    game_data.continue_on_command = false;
    let game_mock = GameMock::new(&mut game_data, 1000);
    let mut game_server = create_server(game_mock, "3333".into())?;
    game_server.run();
    let stop = timer.elapsed();
    assert!(stop - start < std::time::Duration::from_millis(100));
    Ok(())
}

#[test]
fn server_should_stop_if_handle_event_returns_false() -> Result<(), Exception> {
    let timer = std::time::Instant::now();
    let start = timer.elapsed();
    let mut game_data = GameData::new();
    game_data.continue_on_event = false;
    let game_mock = GameMock::new(&mut game_data, 1000);
    let mut game_server = create_server(game_mock, "3334".into())?;
    game_server.run();
    let stop = timer.elapsed();
    assert!(stop - start < std::time::Duration::from_millis(100));
    Ok(())
}

#[test]
fn server_should_recv_commands_from_client() -> Result<(), Exception> {
    std::thread::spawn(|| {
        let res = ClientSocket::new("1111", "127.0.0.1:3335")
            .map(|mut c| {
                for _i in 0..1000 {
                    c.send(vec![1u8, 3u8]);
                }
                1
            })
            .unwrap_or(0);
        res
    });

    let mut game_data = GameData::new();
    let game_mock = GameMock::new(&mut game_data, 100);
    let mut game_server = create_server(game_mock, "3335".into())?;
    game_server.run();
    //  assert!(t.join().unwrap_or(0) > 0);
    assert!(
        game_data
            .updates
            .iter()
            .any(|(_, y, _)| y.iter().any(|v| *v == vec![1u8, 3u8])),
        "len {}",
        game_data.updates.len()
    );
    Ok(())
}

trait Middleware<T> {
    fn execute(&mut self, data: T) -> Result<T, Box<Error>>;
    fn next(&mut self) -> &mut Option<Box<dyn Middleware<T>>>;
    fn run(&mut self, data: T) -> Result<T, Box<Error>> {
        let data = self.execute(data)?;
        match &mut self.next() {
            Some(next) => next.execute(data),
            None => Ok(data),
        }
    }
}

fn compose<T: From<U>, U>(
    rhs: impl FnOnce(T) -> U,
    lhs: impl FnOnce(T) -> U,
) -> impl FnOnce(T) -> U {
    move |x| lhs(rhs(x).into())
}

fn curry<T, U, Z>(x: T, f: impl FnOnce(T, U) -> Z) -> impl FnOnce(U) -> Z {
    move |y| f(x, y)
}

fn add(x: i32, y: i32) -> i32 {
    x + y
}

fn add_static(mut x: i32) -> impl FnMut(i32) -> i32 {
    move |y| {
        x += 10;
        x + y
    }
}

struct AddOne {
    next: Option<Box<Middleware<i32>>>,
}

impl Middleware<i32> for AddOne {
    fn execute(&mut self, data: i32) -> Result<i32, Box<Error>> {
        Ok(data + 1)
    }

    fn next(&mut self) -> &mut Option<Box<Middleware<i32>>> {
        &mut self.next
    }
}

enum Operation {
    Add,
    Mul,
}

struct Calculator<T> {
    pub op: Operation,
    pub lhs: T,
    pub rhs: T,
    pub result: Option<T>,
}

impl<'a, 'b: 'a, T: 'b + Add<Output = T> + Mul<Output = T> + Borrow<T>> Calculator<T>
where
    &'a T: Add<Output = T> + Mul<Output = T>,
{
    pub fn calculate_procedurally(&'b mut self) {
        let res: T = match self.op {
            Operation::Add => &self.lhs + &self.rhs,
            Operation::Mul => &self.lhs * &self.rhs,
        };
        self.result = Some(res);
    }
}

impl<T: Add<Output = T> + Mul<Output = T> + Clone> Calculator<T> {
    pub fn calculate_functionally(mut self) -> Self {
        self.result = Some(match self.op {
            Operation::Add => self.lhs.clone() + self.rhs.clone(),
            Operation::Mul => self.lhs.clone() * self.rhs.clone(),
        });
        self
    }
}

#[cfg(test)]
mod example {
    #[test]
    fn test_add() {
        let mut calc = self::super::Calculator::<isize> {
            op: self::super::Operation::Add,
            lhs: 2,
            rhs: 3,
            result: None,
        };
        calc = calc.calculate_functionally();
        assert_eq!(5, calc.result.unwrap());
    }

    #[test]
    fn test_mul() {
        let mut calc = self::super::Calculator::<isize> {
            op: self::super::Operation::Mul,
            lhs: 2,
            rhs: 3,
            result: None,
        };
        calc = calc.calculate_functionally();
        assert_eq!(6, calc.result.unwrap());
    }

    #[test]
    fn test_add_proc() {
        let mut calc = self::super::Calculator::<isize> {
            op: self::super::Operation::Add,
            lhs: 2,
            rhs: 3,
            result: None,
        };
        calc.calculate_procedurally();
        assert_eq!(5, calc.result.unwrap());
    }

    #[test]
    fn test_mul_proc() {
        let mut calc = self::super::Calculator::<isize> {
            op: self::super::Operation::Mul,
            lhs: 2,
            rhs: 3,
            result: None,
        };
        calc.calculate_procedurally();
        assert_eq!(6, calc.result.unwrap());
    }
}
