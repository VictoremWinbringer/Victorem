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

use std::ops::{Add, Mul};
use std::borrow::Borrow;

enum Operation {
    Add,
    Mul,
}

struct Calculator<T> where {
    pub  op: Operation,
    pub    lhs: T,
    pub  rhs: T,
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