# Koto - Music performing filesystem

![koto logo](koto-logo.gif)

*Koto* is a music performing filesystem, or a filesystem (sound) synthesizer. *Koto* allows to perform sound and music by interaction with mounted FUSE filesystem. If you don't make sense with it, check this demo movies below.

- [filesystem performance](https://www.youtube.com/watch?v=Rxh-msWrj6o)
- [filesystem performance 2](https://youtube.com/watch?v=dV0xoK5ARfI)
- [simple performance (without filesystem feature)](https://www.youtube.com/watch?v=W_rGWa86TZg)

*Koto* uses and depends on [Tapirus](https://github.com/t-sin/tapirus) sound synthesizer modules.

[Dedicated](https://twitter.com/tobuzoo7/status/999061314818359296) [with](https://twitter.com/tobuzoo7/status/1021623019465146368) [you](https://mobile.twitter.com/sin_clav/status/1069065073912496130).

## Requirements

### GNU/Linux

- libasound (ALSA)
- libfuse (>= 2.6.0)

### macOS

- osxfuse

## Installation

Pre-built binaries are available from [here](https://github.com/t-sin/koto/releases).

If you want to build, clone and type `cargo build`.

## Usage

### Command line usage

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

When you think about to stop performace with *Koto*, you might want to save current configuration, to resume performance after like drinking a cup of tea. It's times like these, you can save entire synthesizer configuration (includeing sequencer pattern and wavetable values). To save the configuration, send a `SIGUSR1` signal to the running *Koto* process, so a file `koto.yyyymmddThhmmss.lisp` is created in the directory placed *Koto* binary.

### Basic concepts of Koto

Koto is a real-time sound processing system and we can interact via user interface. The UI is a filesystem. Koto has sound processing modules in it, these construct a graph that has a root as speaker output. Each sound processing modules, are like oscillators, effects or sequencers, have some parameters (e.g. delay time, etc.).

In Koto, one sound module are mapped as a directory. And, one parameter of sound module are mapped as a file or a diretory in the sound module directory.

### Interact with Koto

Let's see some examples.

If you launch Koto with no configuration file, Koto mount specified directory, then start sound processing with no sound.

```sh
$ koto ./mountpoint
$ ls ./mountpoint
src0.val  vol.val
$ cat ./mountpoint/src0.val ; echo
0
$ cat ./mountpoint/vol.val ; echo
0
```

This is initial configuration. In this case, the root module has two parameters: `src0` and `vol`. File extension `.val` tells to Koto that "it's a static value." In Koto, filename is important thing to know what kind of sound module the file/directory is. `vol.val` is zero, so we should change this value. To change value, simply write, like this:

```sh
$ cat ./mountpoint/vol.val ; echo
0
$ echo 0.3 > ./mountpoint/vol.val
$ cat ./mountpoint/vol.val ; echo
0.3
```

`vol.val` is changed but still no sound. So let's play sine wave. Because this filesystem is a user interface, we can add sine wave with manipulating filesystem. So we will delete `src0.val`, create a directory named `src0.sine`, set frequency as a file and request updation to Koto. Like this:

```sh
$ cd ./mountpoint
# delete value module
$ rm src0.val
# create sine module (but not connected)
$ mkdir src0.sine
# set frequency of sine module
$ echo 440 > src0.sine/freq.val
# tell koto a request to connect sine module
$ touch src0.sine/
```

Now we have 440 Hz sine wave.

### Sound modules

Koto has some sound modules. Here is a list of modules and its parameters.

#### Oscillators

- `rand`: noise generator
    - `freq`: a sample time to renew output value.
- `sine`: sine wave generator
    - `init_ph`: initial phase of oscillator. for phase distortion synthesis.
    - `freq`: a frequency of this oscillator
- `tri`: triangle wave generator
    - `init_ph`: initial phase of oscillator. for phase distortion synthesis.
    - `freq`: a frequency of this oscillator
- `saw`: saw wave generator
    - `init_ph`: initial phase of oscillator. for phase distortion synthesis.
    - `freq`: a frequency of this oscillator
- `pulse`: pulse (square) wave generator
    - `init_ph`: initial phase of oscillator. for phase distortion synthesis.
    - `freq`: a frequency of this oscillator.
    - `duty`: duty ratio.
- `phase`: phase ocsillator utility for wavetable
    - it restrict input in range of [0.0, 1.0]
    - `osc`: phase source.
- `wavetable`: wave table oscillator
    - NOTE: now this module interporate linear between table samples.
    - `table`: wave table.
    - `ph`: phase source.

#### Sequencers

- `trig`: envelope trigger
    - `eg`: main envelove generator. `trig` returns this module's value.
    - `srcN`: other envelove generators triggered by `trig`. output is discarded. rest parameter.
- `adsr`: ADSR envelop generator
    - `a`: attack value in sec.
    - `d`: decay value in sec.
    - `s`: sustin value in range of [0.0, 1.0].
    - `r`: release value in sec.
- `seq`: Sequencer
    - `pattern`: sequencer pattern.
    - `osc`: oscillator.
    - `osc_mod`: frequency modulator for the `osc`.
    - `eg`: envelope generator. it's triggered by `seq`.

#### Effects

- `lpf`: simple low-pass filter
    - `freq`: cutoff frequency.
    - `q`: filter resonance.
    - `src`: filter source.
- `delay`: mono delay
    - `time`: delay time in sec.
    - `feedback`: feedback volume in percent.
    - `mix`: mix in percent.
    - `src`: delay source.

#### Utilities

- `pan`: panning utility
    - `pan`: pan value. left is -1.0, right is 1.0.
    - `src`: source.
- `clip`: clipping utility
    - `min`: min value.
    - `max`: max value.
    - `src`: source.
- `offset`: offset utility
    - `val`: base value
    - `src`: source.
- `gain`: gain utility
    - `gain`: gain multiplyer
    - `src`: source.
- `+`: signal addition utility
    - `srcN`: signal sources. rest parameter.
- `*`: signal multiplying utility
    - `srcN`: signal sources. rest parameter.
- `out`: mixing utility with volume
    - `vol`: volume
    - `srcN`: signal sources. rest parameter.

## Author

- TANAKA Shinichi (<shinichi.tanaka45@gmail.com>)

## License

This program *koto* is licensed under the GNU General Public License Version 3. See [LICENSE](LICENSE) for details.
