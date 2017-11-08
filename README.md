# osmium

[![Build Status](https://travis-ci.org/ThetaSinner/osmium.svg?branch=master)](https://travis-ci.org/ThetaSinner/osmium)

An http2 server

### Developer setup instructions

- Make sure you have rustc 1.21 (or above)
- Install OpenSSL 1.0.2 (or above)
- On some systems it might be enough to add OpenSSL to the path. However, setting the variable OPENSSL_DIR to the root directory of the OpenSSL   install (i.e. not the /bin subdirectory)
- Clone [this repository](https://github.com/ThetaSinner/osmium)
- run `cargo build` in the project root

You'll probably see some warnings, but as long as there are no errors you're ready to work on osmium, or import it into another project.