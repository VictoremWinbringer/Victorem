#[macro_use] extern crate serde_derive;

use bincode::{serialize, deserialize};
use serde;

mod entities;
mod data_access_layer;
mod for_tests {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
   pub struct TestData {
       pub counter: i32
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
            let mut d = 1i32;
         d_mut(&mut d);
         d_not_mut(&d);
        d_mut(&mut d);
        d_not_mut(&d);
        d_mut(&mut d);
        d_not_mut(&d);
        d_mut(&mut d);
        d_not_mut(&d);
        let test_data = crate::for_tests::TestData{counter:2};
        let data = bincode::serialize(&test_data).unwrap();
        let res:crate::for_tests::TestData = bincode::deserialize(&data).unwrap();
        crate::data_access_layer::logger::init();
        crate::data_access_layer::logger::error("lasjdflajslfdasdf");

        assert_eq!(res.counter, test_data.counter);
    }

    fn d_mut(d:&mut i32){

    }

    fn d_not_mut(d: &i32){

    }
}
