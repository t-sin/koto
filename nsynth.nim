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

  const framesPerBuffer = 512
  const freqency = 3
  let table = makeTable(framesPerBuffer, sin)

  type
    TState = tuple[n: float32]
    TStereo = tuple[left, right: float32]

  proc fillingWithTable(inBuf, outBuf: pointer,
                        framesPerBuf: culong,
                        timeInfo: ptr TStreamCallbackTimeInfo,
                        stateusFlags: TStreamCallbackFlags,
                        userData: pointer): cint {.cdecl.} =
    var
      outBuf = cast[ptr array[framesPerBuffer, TStereo]](outBuf)
      state = cast[ptr TState](userData)

    proc linear_interpolate(x: float32): float32 =
      let
        xprev = int(m.floor(x)) mod table.len
        xnext = int(m.ceil(x)) mod table.len
      return table[xprev] + table[xnext] / 2

    for i in 0..<int(framesPerBuf):
      let
        x = float32(i) * freqency + state.n
        val = linear_interpolate(x)
      outBuf[i] = (val, val)

    state.n = float32(framesPerBuf) * freqency + state.n

  var
    stream: PStream
    state = (n: 0)

  discard PA.OpenDefaultStream(
    cast[PStream](stream.addr),
    numInputChannels = 0,
    numOutputChannels = 2,
    sampleFormat = sfFloat32,
    sampleRate = 44_100,
    framesPerBuffer = 256,
    streamCallback = fillingWithTable,
    userData = cast[pointer](state.addr))

  discard PA.StartStream(stream)
  PA.Sleep(2000)
  discard PA.StopStream(stream)
  discard PA.CloseStream(stream)

  echo "============== terminate pa   ==============="
  echo repr(PA.Terminate())
