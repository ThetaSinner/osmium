# osmium

[![Build Status](https://travis-ci.org/ThetaSinner/osmium.svg?branch=master)](https://travis-ci.org/ThetaSinner/osmium)

An http2 server

### Change Wishlist
- Use the tls-api so that Osmium users can choose their tls implementation. At the moment Osmium is completely tied to OpenSSL. 
    + Waiting for tls-api to be implemented by the Rust developers.
- Connection shutdown currently requires the use of a boolean flag to guard against new frames being queued for processing and queued promises having their execution initiated. It would be much cleaner to perform shutdown actions in the connection then use an exception to break the read/write loop. 
    + Waiting for exceptions to be implemented in Rust, there's an RFC but nothing on stable yet.
- Ability to run streams on seperate threads. Currently, each connection is assigned a thread and all connection and stream processing occur on that single thread.
    + The number of connections should easily outgrow the number of threads the host has to offer. This means that the whole concurrency model has to change to make this work. The connections need to not hold onto a thread when they're innactive, so that the thread can be used by another connection. That or a lighweight threadpool needs to be used.
    + Some intelligence needs to go into assigning resources to a connection. Probably the most logical way to go about this would be to implement priority, and use that as a best guess to decide when a connection can take advantage of concurrent streams vs a connection which is just making a lot of requests independently. A more sophisticated system than that would have to be both another wishlist item and priority would need to be seen to be more commonly used than it currently is.
- Create a server administration tool that allows server/stream state to be viewed in real time with history etc. This is part of a more technical requirement that connections be made subject to administration by the main thread, so that suspected idle connections can be pinged and shutdown if necessary.
- Setup benches to performance test the server, and test coverage to provide confidence. Both of these are difficult because of the slightly restrictive way that tests have to be run in rust. To do these properly a seperate process needs to be started for the server, but data can't be collected about the seperate process.

### Developer setup instructions

- Make sure you have rustc 1.21 (or above)
- Install OpenSSL 1.0.2 (or above)
- On some systems it might be enough to add OpenSSL to the path. For something that should work everywhere, setting the variable OPENSSL_DIR to the root directory of the OpenSSL install (i.e. not the /bin subdirectory)
- Clone [this repository](https://github.com/ThetaSinner/osmium)
- run `cargo build` in the project root

You'll probably see some warnings, but as long as there are no errors you're ready to work on osmium, or import it into another project.
