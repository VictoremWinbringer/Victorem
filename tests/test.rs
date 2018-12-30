use std::error::Error;
use victorem;


#[test]
fn it_works() {
    let mut id = AddOne { next: Some(Box::new(AddOne { next: None })) };
    let data = id.run(3);
    let f = curry(1,add);
    let f = compose(f,curry(1,add));
    let data = f(3);
        assert!(false, "{:?}", data);
}

#[test]
fn static_add(){
  let mut f = add_static(3);
    let data = format!("one {}, two {}, three {}",f(1),f(1),f(1));
    let r:Vec<i32> = (1..=3).collect();
    assert!(false, "{:#?}", r);
}

trait Middleware<T> {
    fn execute(&mut self, data: T) -> Result<T, Box<Error>>;
    fn next(&mut self) -> &mut Option<Box<dyn Middleware<T>>>;
    fn run(&mut self, mut data: T) -> Result<T, Box<Error>> {
        let data = self.execute(data)?;
        match &mut self.next() {
            Some(next) => next.execute(data),
            None => Ok(data),
        }
    }
}

fn compose<T:From<U>, U>(rhs: impl FnOnce(T) -> U, lhs: impl FnOnce(T) -> U) -> impl FnOnce(T) -> U {
   move |x| lhs(rhs(x).into())
}

fn curry<T, U, Z>(x: T, f: impl FnOnce(T, U) -> Z) -> impl FnOnce(U) -> Z {
   move |y| f(x, y)
}

fn add(x: i32, y: i32) -> i32 {
    x + y
}
fn add_static(mut x:i32) -> impl FnMut(i32)->i32{
     move |y|{
          x +=10;
          x+y
      }
}
struct AddOne {
    next: Option<Box<Middleware<i32>>>
}

impl Middleware<i32> for AddOne {
    fn execute(&mut self, mut data: i32) -> Result<i32, Box<Error>> {
        Ok(data + 1)
    }

    fn next(&mut self) -> &mut Option<Box<Middleware<i32>>> {
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