# Rustmania
> Its a VSRG


RustMania is a Vertically Scrolling Rhythm Game heavily inspired by Stepmania and Etterna.

Why you should by excited about RustMania:

* Rust is fast and safe
* New engine free from legacy code
* Completely open source
* Compatible with .sm files
* Simple system for creating Noteskins
* Cross platform

Cool things that are planned:
* Support for reading from and writing to all common simfile formats, including a custom format for Rustmania.
* Support for a variety of audio formats
* Lua scripting
* Editor
* Difficulty calc
* Potential to play in browser
* 

![](header.png)

## Installation

To compile RustMania, you will first need to install Rust, this can most easily be done through `rustup`. 
On Windows, you can download and run the installer [here](https://www.rust-lang.org/en-US/install.html).
On Linux or macOS, you can install rustup with the following command: 

```curl https://sh.rustup.rs -sSf | sh```

Once Rust is installed, you can compile and run the game using `Cargo`, Rust's built-in package manager.
To compile and open the resulting binary with the default resources, you can simply input the following command:

```
cargo +nightly run --release Songs/Mu/mu.sm Default resources/script.lua
```

## Usage example

A few motivating and useful examples of how your product can be used. Spice this up with code blocks and potentially more screenshots.

_For more examples and usage, please refer to the [Wiki][wiki]._

## Development setup

Learn Rust and Lua lmoa

## Release History

Next Release
* 0.1.0
    * Work in progress

## Meta

ixsetf :)

## Licence

Distributed under the MIT license. See ``LICENSE`` for more information.

"Mu" by Solarbear is licensed under CC SA 3.0


## Contributing

1. Fork it (<https://github.com/Rgates94/Rustmania/fork>)
2. Create your feature branch (`git checkout -b feature/fooBar`)
3. Commit your changes (`git commit -am 'Add some fooBar'`)
4. Push to the branch (`git push origin feature/fooBar`)
5. Create a new Pull Request

<!-- Markdown link & img dfn's -->
[npm-image]: https://img.shields.io/npm/v/datadog-metrics.svg?style=flat-square
[npm-url]: https://npmjs.org/package/datadog-metrics
[npm-downloads]: https://img.shields.io/npm/dm/datadog-metrics.svg?style=flat-square
[travis-image]: https://img.shields.io/travis/dbader/node-datadog-metrics/master.svg?style=flat-square
[travis-url]: https://travis-ci.org/dbader/node-datadog-metrics
[wiki]: https://github.com/yourname/yourproject/wiki

