# Rustmania

<p align="center">
  <a href="https://github.com/rustmaniagame/rustmania/actions?query=workflow%3ACI" alt="CI">
    <img src="https://github.com/rustmaniagame/rustmania/workflows/CI/badge.svg" />
  </a>
  <a href="https://dependabot.com/" alt="Dependabot">
    <img src="https://badgen.net/dependabot/rustmaniagame/rustmania/?icon=dependabot" />
  </a>
  <a href="https://github.com/rustmaniagame/rustmania/blob/master/LICENSE" alt="License">
    <img src="https://badgen.net/badge/license/MIT/blue" />
  </a>
</p>

RustMania is a Vertically Scrolling Rhythm Game inspired by Stepmania and Etterna.

Why you should by excited about RustMania:

* Rust is fast and safe
* New engine free from legacy code
* Support for a variety of audio formats
* Completely open source
* Compatible with .sm files
* Simple system for creating Noteskins
* Cross platform

Cool things that are planned:
* Support for reading from and writing to all common simfile formats, including a custom format for Rustmania.
* Lua scripting
* Simfile editor
* Automatic difficulty calculator
* Potential to play in browser
* ...and more!

## Installation

To compile RustMania, you will first need to install Rust, this can most easily be done through `rustup`. 

On Windows, you can download and run the installer [here](https://www.rust-lang.org/en-US/install.html).

On Linux or macOS, you can install rustup with the following command: 

```curl https://sh.rustup.rs -sSf | sh```

Once Rust is installed, you can compile and run the game using `Cargo`, Rust's built-in package manager.
To compile and open the resulting binary with the default resources, you can simply input the following command:

```
cargo +nightly run --release
```

## Release History

Next Release
* 0.1.0
    * Work in progress

## Licence

Distributed under the MIT license. See ``LICENSE`` for more information.

"Mu" by Solarbear is licensed under CC SA 3.0


## Contributing

1. Fork it (<https://github.com/Rgates94/Rustmania/fork>)
2. Create your feature branch (`git checkout -b feature/fooBar`)
3. Commit your changes (`git commit -am 'Add some fooBar'`)
4. Push to the branch (`git push origin feature/fooBar`)
5. Create a new Pull Request
