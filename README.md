# Koto - Music performing filesystem

![koto logo](koto-logo.gif)

[Dedicated](https://twitter.com/tobuzoo7/status/999061314818359296) [with](https://twitter.com/tobuzoo7/status/1021623019465146368) [you](https://mobile.twitter.com/sin_clav/status/1069065073912496130).

## Demo

- [simple performance (without filesystem feature)](https://www.youtube.com/watch?v=W_rGWa86TZg)
- [filesystem performance](https://www.youtube.com/watch?v=Rxh-msWrj6o)

## TODOs

- [x] Unit graph framework
    - [x] frameworks
    - [x] zero, offset, gain
    - [x] stereonize (pan)
- [x] Oscillators
    - [x] Sine, Triangle, Saw, Square
    - [x] Wavetable
- [x] Effects
    - [x] Filters
    - [x] Delay
- [x] Sequencers
    - [x] Envelope
    - [x] Events (notes, fx values)
    - [x] Sequencers
    - [x] MML-like score description
- [x] SAVE the graph
    - [x] Lisp reader/printer
    - [x] Construct unit graph (with cheap `eval`)
    - [x] Global binding and `def` (with unit sharing)
    - [x] SAVE the graph
- [x] FUSE interfaces
    - [x] Simple in-memory filesystem
    - [x] Map unit graph into filesystem (read only)
    - [x] Unit manipulation via filesystem
- [ ] Command line interfaces
    - [x] initial configuration
    - [x] mountpoint
    - [ ] dump current configuration by signals
- [ ] Module & Crate separating
    - Tapirus (unit generators & TapirLisp)
    - KFS (types, KotoNode, Filesystem trait)
- [ ] Documentation
    - Koto concept
    - Sound modules
    - Filesystem representations
    - Configurations and Tapir Lisp

## Requirements

### GNU/Linux

- pkg-config
- libasound (ALSA)
- libfuse (>= 2.6.0)

### macOS

- FUSE for macOS

## Installation

Upcomming...

## Usage

To launch *Koto*, type like this:

```sh
$ /path/to/koto /path/to/mountpoint
```

### Tapir Lisp

Tapir Lisp is a Lisp-like language for describing synthesizer signal flow.
It's used to save and load synthesizer configuration that created interactively by a user.
For details, see [Tapir Lisp's README](src/tapirlisp/README.md).

## Author

- TANAKA Shinichi (<shinichi.tanaka45@gmail.com>)

## License

This program *koto* is licensed under the GNU General Public License Version 3. See [LICENSE](LICENSE) for details.
