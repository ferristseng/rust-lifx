# rust-lifx

## Description

`rust-lifx` provides a simple implementation of a client that can talk to any
number of lifx bulbs, and provides an implementation of the payload, header, 
messages, and serialization procedures described in the LAN protocol on the 
LiFX developer webpage.

Currently, the implementation targets `V2.0` of the LAN protocol.

[Full Protocol](http://lan.developer.lifx.com/docs/)

## Contributing

Everyone is encouraged to contribute! Please create a pull request if you want 
to add to or modify the code!

### Setup

`cargo build` will build the library and all of the examples. 
`rust-lifx` tracks the nightly branch of Rust, so that it can use the 
most up-to-date `net2` features.

```
  cargo build
```

### Configure the logger to print while running examples

To configure the logger to print out useful information while running the examples
run:

```
  source ./config_logger.sh
```

For more detailed control, you the following loggers you can set:

  * device.in - all messages from any device that the client can intercept 
  * device.out - all output messages from the client

# License

The MIT License (MIT)

Copyright (c) <2015> <Ferris Tseng>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
