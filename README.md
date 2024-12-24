# iostub
iostub is a small and ready to use rust library to stub std::io::Read in test suites.

## usage

IOStub allow you to provide a object implementing std::io::Read which can be consumed by your test subject
and still allow you to push data or errors trough it.
```rust
        let mut stub = IOStub::new();
        let mut cr = ConsumeReader::new(stub.clone());

        stub.push_read(b"otters");
        stub.push_read(b"are");
        stub.push_read(b"amazing");
        let rv = cr.read_all();
        assert!(rv.is_ok());
        assert_eq!(rv.unwrap(), Vec::from(b"ottersareamazing"));

        let rv = cr.read_all();
        assert!(rv.is_ok());
        assert_eq!(rv.unwrap(), Vec::from(b""));

        stub.push_read(b"from");
        stub.push_read(b"otter");
        stub.push_read(b"space");
        let rv = cr.read_all();
        assert!(rv.is_ok());
        assert_eq!(rv.unwrap(), Vec::from(b"fromotterspace"))

        stub.push_read_error(Error::new(ErrorKind::TimedOut, "xxx"));
        let rv = cr.read_one();
        assert!(rv.is_err());
        let e = rv.unwrap_err();
        assert_eq!(e.kind(), ErrorKind::TimedOut);
```

### caution
This library was designed for testing purposes and is not optimised for anything else.
