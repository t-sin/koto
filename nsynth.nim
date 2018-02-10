from os import commandLineParams

import vorbis/vorbisfile as VF
import portaudio as PA

let args = commandLineParams()

if args.len == 1:
  var vf: VF.TOggVorbis_File

  if not VF.fopen(args[0], vf.addr) == 0:
    echo "cannot open '" & args[0] & "'"
    quit(1)

  echo repr(VF.info(vf.addr, -1))

  echo repr(PA.Initialize())

  echo repr(PA.Terminate())
