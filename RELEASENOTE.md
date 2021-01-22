# Koto - Music performing filesystem

![koto logo](koto-logo.gif)

All changelogs are [here](CHANGELOG.md).

## Version 1.1.0

Welcome to new version of Koto!

This release has many changes but notable topics are below:

- Fix sequencer behavior
    - before this, sequencers with long sequence are buggy.
    - this bugfix is fixed in [Tapirus](https://github.com/t-sin/tapirus) sound synthesizer module.
- Refine demo music
    - add some comments to [configure.lisp](configure.lisp)
    - add new demo song; Donald Byrd's [fancy-free.lisp](fancy-free.lisp)
- Add some shell scripts for utilize Koto
    - it includes a script make wavetable oscillator a sampler
