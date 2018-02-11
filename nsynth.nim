from os import commandLineParams
import math as m

import vorbis/vorbisfile as VF
import portaudio as PA


proc makeTable(size: int, fn: proc (n: int, max: int): float32): seq[float32] =
  result = newSeq[float32](size)
  for i in 0..<size:
    result[i] = fn(i, size)

proc sin(n: int, max: int): float32 =
  let i = float(n) / float(max)
  result = m.sin(i * 2 * m.PI)


let args = commandLineParams()

if args.len == 1:
  var vf: VF.TOggVorbis_File

  if not VF.fopen(args[0], vf.addr) == 0:
    echo "cannot open '" & args[0] & "'"
    quit(1)

  echo "============== show .ogg info ==============="
  echo "filename: '" & args[0] & "'"
  echo repr(VF.info(vf.addr, -1))

  echo "============== initialize pa  ==============="
  echo repr(PA.Initialize())

  proc streamCallback (inBuf, outBuf: pointer,
                       framesPerBuf: culong,
                       timeInfo: ptr TStreamCallbackTimeInfo,
                       stateusFlags: TStreamCallbackFlags,
                       userData: pointer): cint =
    discard
  # var stream: PStream

  # PA.OpenDefaultStream(
  #   stream.addr,
  #   numInputChannels = 0,
  #   numOutputChannels = 2,
  #   sampleFormat = sfFloat32,
  #   sampleRate = 44_100,
  #   framesPerBuffer = 256,
  #   streamCallback = cast[pointer](0),
  #   userData = cast[pointer](0))

  echo "============== terminate pa   ==============="
  echo repr(PA.Terminate())
