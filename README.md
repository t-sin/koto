# Koto - Music performing filesystem

![koto logo](koto-logo.gif)

*Koto* is a music performing filesystem, or a filesystem (sound) synthesizer. *Koto* allows to perform sound and music by interaction with mounted FUSE filesystem. If you don't make sense with it, check this demo movies below.

- [filesystem performance](https://www.youtube.com/watch?v=Rxh-msWrj6o)
- [filesystem performance 2](https://youtube.com/watch?v=dV0xoK5ARfI)
- [simple performance (without filesystem feature)](https://www.youtube.com/watch?v=W_rGWa86TZg)

*Koto* uses and depends on [Tapirus](https://github.com/t-sin/tapirus) sound synthesizer modules.

[Dedicated](https://twitter.com/tobuzoo7/status/999061314818359296) [with](https://twitter.com/tobuzoo7/status/1021623019465146368) [you](https://mobile.twitter.com/sin_clav/status/1069065073912496130).

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
- [x] Command line interfaces
    - [x] initial configuration
    - [x] mountpoint
    - [x] dump current configuration by signals
- [x] Module & Crate separating
    - [x] Tapirus (unit generators & TapirLisp)
    - [x] KFS (types, KotoNode, Filesystem trait)
- [ ] Documentation
    - [ ] Koto concept
    - [ ] Sound modules
    - [ ] Filesystem representations
    - [ ] Configurations and Tapir Lisp

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

### Basic usage

Koto uses FUSE filesystem as user interface, so, we should create mountpoint and mount it at first. To start and stop *Koto* simply, type like this:

```sh
# start koto
$ /path/to/koto /path/to/mountpoint

# stop koto
$ fusermount -u /path/to/mountpoint
```

Koto takes one positional arg `MOUNTPOINT` and some options. Important option in those is `-c` (`--config`). This specified initial configuration of synthesizer modules. Because of this *Koto* can start music by specified configuration. If you try it, type like this:

```sh
$ cd /path/to/koto
$ ./koto /path/to/mountpoint -c ./configure.lisp
```

When you think about to stop performace with *Koto*, you might want to save current configuration, to resume performance after like drinking a cup of tea. It's times like these, you can save entire synthesizer configuration (includeing sequencer pattern and wavetable values). To save the configuration, send a `SIGUSR1` signal to the running *Koto* process, so a file `koto.yyyymmddThhmmss.lisp` is created in the directory placed *Koto* binary. Note that **this feature has some problems** (TapirLisp in Tapirus does not support `;` comments, or dumped config cannot be load, and so on).

### Intaract with Koto

Upcomming...

## Author

- TANAKA Shinichi (<shinichi.tanaka45@gmail.com>)

## License

This program *koto* is licensed under the GNU General Public License Version 3. See [LICENSE](LICENSE) for details.
