# Package

version       = "0.1.0"
author        = "TANAKA Shinichi"
description   = "A synthesizer and sequencer library written with Nim."
license       = "GPL-3.0"
srcDir        = "src"
bin           = @["koto"]

# Dependencies

requires "nim >= 0.19.0"
requires "nim-portaudio >= 0.1.3"
