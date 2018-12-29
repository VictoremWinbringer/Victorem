use std::error::Error;
use victorem;


#[test]
fn it_works() {

    let mut id = IdMiddleware::new(Some(Box::new(IdMiddleware2::new(None))));
    let mut data = vec![1u8, 2u8, 3u8, 4u8];
    let data = id.run(&mut data);
    assert!(false, "{:?}", data);
}

trait Middleware<T: ?Sized> {
    fn execute<'a: 'b, 'b>(&mut self, data: &'a mut T) -> Result<&'b mut T, Box<Error>>;
    fn next(&mut self) -> &mut Option<Box<dyn Middleware<T>>>;
    fn run<'a: 'b, 'b>(&mut self, data: &'a mut T) -> Result<&'b mut T, Box<Error>> {
        let data = self.execute(data)?;
        match &mut self.next() {
            Some(next) => next.execute(data),
            None => Ok(data),
        }
    }
}

struct IdMiddleware {
    next: Option<Box<dyn Middleware<[u8]>>>,
}

impl IdMiddleware {
    fn new(next: Option<Box<dyn Middleware<[u8]>>>) -> IdMiddleware {
        IdMiddleware { next }
    }
}

impl Middleware<[u8]> for IdMiddleware {
    fn execute<'a: 'b, 'b>(&mut self, data: &'a mut [u8]) -> Result<&'b mut [u8], Box<Error>> {
        let len = data.len();
        let data = &mut data[1..len];
        Ok(data)
    }

    fn next(&mut self) -> &mut Option<Box<Middleware<[u8]>>> {
        &mut self.next
    }
}

struct IdMiddleware2 {
    next: Option<Box<dyn Middleware<[u8]>>>,
}

impl IdMiddleware2 {
    fn new(next: Option<Box<dyn Middleware<[u8]>>>) -> IdMiddleware2 {
        IdMiddleware2 { next }
    }
}

impl Middleware<[u8]> for IdMiddleware2 {
    fn execute<'a: 'b, 'b>(&mut self, data: &'a mut [u8]) -> Result< &'b mut [u8], Box < Error >> {
        let len = data.len();
    let data = &mut data[1..len];
    Ok(data)
    }

    fn next(&mut self) -> &mut Option<Box<Middleware<[u8]>>> {
        &mut self.next
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