# Changelogs

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

## Version 1.0.0

🎉🎊🎉🎊🎉🎊🎉🎊🎉🎊🎉🎊🎉🎊🎉🎊🎉🎊🎉🎊🎉🎊🎉🎊

It's first release! Congrats!

- Comments are available in TapirLisp code
- Fix variable definition order problem

These iprovement are fixed in [Tapirus](https://github.com/t-sin/tapirus) sound synthesizer module.

## Version 0.9.0

Pre-release for v1.0.0.

- Implement sound processing modules (see [Tapirus](https://github.com/t-sin/tapirus))
- Load synthesizer module configuration from TapirLisp
- Dump synthesizer module configuration into TapirLisp
- Filesystem as a user interface
- Automated release with GitHub Actions
