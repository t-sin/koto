# Package

version       = "0.1.0"
author        = "TANAKA Shinichi"
description   = "A synthesizer and sequencer library written with Nim."
license       = "GPL-3.0"
srcDir        = "src"
bin           = @["primitive"]

# Dependencies

requires "nim >= 0.17.2"
requires "vorbis >= 0.1.3"
requires "nim-portaudio >= 0.1.3"