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
    continue_running: bool,
    disconnect_this_client: Option<SocketAddr>,
    draw: Vec<u8>,
    drawn: Vec<Duration>,
    new_client: Option<SocketAddr>,
    disconnect_on_event: bool,
}

impl GameData {
    fn new() -> GameData {
        GameData {
            events: Vec::new(),
            updates: Vec::new(),
            continue_running: true,
            disconnect_this_client: None,
            draw: Vec::new(),
            drawn: Vec::new(),
            new_client: Some(SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                7777,
            )),
            disconnect_on_event: false,
        }
    }
}

struct GameMock<'a> {
    data: &'a mut GameData,
}

impl<'a> GameMock<'a> {
    fn new(data: &'a mut GameData) -> GameMock {
        GameMock { data }
    }
}

impl<'a> Game for GameMock<'a> {
    fn update(&mut self, delta_time: Duration, commands: Vec<Vec<u8>>, from: SocketAddr) -> bool {
        self.data.updates.push((delta_time, commands, from));
        self.data.continue_running
    }

    fn draw(&mut self, delta_time: Duration) -> Vec<u8> {
        self.data.drawn.push(delta_time);
        self.data.draw.clone()
    }

    fn handle_server_event(&mut self, event: ServerEvent) -> ContinueRunning {
        self.data.events.push(event);
        self.data.disconnect_on_event.clone()
    }
    fn add_client(&mut self) -> Option<SocketAddr> {
        self.data.new_client.clone()
    }
}

fn crate_client() -> Result<ClientSocket, Exception> {
    ClientSocket::new("1111", "127.0.0.1:2222")
}

fn create_server(game: GameMock) -> Result<GameServer<GameMock>, Exception> {
    GameServer::new(game, "2222")
}

fn crate_second_client() -> Result<ClientSocket, Exception> {
    ClientSocket::new("3333", "127.0.0.1:2222")
}

#[test]
fn server_works() -> Result<(), Exception> {
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

impl<'a, 'b: 'a, T: 'b + Add<Output=T> + Mul<Output=T> + Borrow<T>> Calculator<T>
    where
        &'a T: Add<Output=T> + Mul<Output=T>,
{
    pub fn calculate_procedurally(&'b mut self) {
        let res: T = match self.op {
            Operation::Add => &self.lhs + &self.rhs,
            Operation::Mul => &self.lhs * &self.rhs,
        };
        self.result = Some(res);
    }
}

impl<T: Add<Output=T> + Mul<Output=T> + Clone> Calculator<T> {
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
