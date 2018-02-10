from os import commandLineParams

import nim-vorbis
import portaudio

let args = commandLineParams()

if args.len == 1:
  echo repr(Initialize())

  echo repr(Terminate())
