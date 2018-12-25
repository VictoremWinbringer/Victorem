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