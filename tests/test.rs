use std::error::Error;
use std::thread;
use threadpool::ThreadPool;
use victorem;

#[test]
fn it_works() {
//    let pool = ThreadPool::new(100);
//    for i in 0..1000 {
//        pool.execute(move || {
//            thread::sleep_ms(100);
//            println!("value {} from thread {:?}", i, thread::current().id());
//        })
//    }
//    thread::sleep_ms(10100);
//    let client = victorem::Client::new("sdfsf", "asdfasf");
    let mut id = IdMiddleware::new(Some(Box::new(IdMiddleware2::new(None))));
    let data = vec![1u8, 2u8, 3u8, 4u8];
    let data = id.execute(&data);
    assert!(false,"{:?}", data);
    assert_eq!(1, 1);
}

fn process(data: &[u8]) -> Result<&[u8], Box<dyn Error>> {
    Ok(&data[..1])
}

trait Middleware {
    fn execute<'a: 'b, 'b>(&mut self, data: &'a [u8]) -> Result<&'b [u8], Box<Error>>;
    fn next() -> &mut Option<Box<dyn Middleware>>;
}

struct IdMiddleware {
    next: Option<Box<dyn Middleware>>,
}

impl IdMiddleware {
    fn new(next: Option<Box<dyn Middleware>>) -> IdMiddleware {
        IdMiddleware { next }
    }
}

impl Middleware for IdMiddleware {
    fn execute<'a: 'b, 'b>(&mut self, data: &'a [u8]) -> Result<&'b [u8], Box<Error>> {
        let len = data.len();
        let data = &data[1..len];
        match &mut self.next {
            Some(next) => next.execute(data),
            None => Ok(data),
        }
    }

    fn next() -> &mut Option<Box<Middleware>> {
        unimplemented!()
    }
}

struct IdMiddleware2 {
    next: Option<Box<dyn Middleware>>,
}

impl IdMiddleware2 {
    fn new(next: Option<Box<dyn Middleware>>) -> IdMiddleware2 {
        IdMiddleware2 { next }
    }
}

impl Middleware for IdMiddleware2 {
    fn execute<'a: 'b, 'b>(&mut self, data: &'a [u8]) -> Result<&'b [u8], Box<Error>> {
        let data = &data[1..data.len()];
        match &mut self.next {
            Some(next) => next.execute(data),
            None => Ok(data),
        }
    }

    fn next() -> &mut Option<Box<Middleware>> {
        unimplemented!()
    }
}

use std::ops::{Add, Mul};
use std::borrow::Borrow;

enum Operation {
    Add,
    Mul,
}

struct Calculator<T> where {
    pub op: Operation,
    pub lhs: T,
    pub rhs: T,
    pub result: Option<T>,
}

impl<'a, 'b: 'a, T: 'b + Add<Output=T> + Mul<Output=T> + Borrow<T>> Calculator<T> where &'a T: Add<Output=T> + Mul<Output=T> {
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
        self.result = Some(
            match self.op {
                Operation::Add => self.lhs.clone() + self.rhs.clone(),
                Operation::Mul => self.lhs.clone() * self.rhs.clone(),
            }
        );
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
        use std::char::*;
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