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

proc saw(n: int, max: int): float32 =
  result = float32(n) / float32(max)

proc tri(n: int, max: int): float32 =
  let
    hperiod = int(max / 2)
    qperiod = int(max / 4)
    croppedN = n mod max

  if n < qperiod:
    result = (float32(n) / float32(qperiod))
  elif n < (hperiod + qperiod):
    result = -(float32(n) / float32(qperiod)) + 2'f32
  else:
    result = float32(n) / float32(qperiod) - 4'f32

proc square(n: int, max: int): float32 =
  let
    hperiod = int(max / 2)
    qperiod = int(max / 4)

  if n < qperiod:
    result = 1'f32
  elif n < (hperiod + qperiod):
    result = 0'f32
  else:
    result = 1'f32


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
  const samplingRate = 44_100
  const freqency = 440
  const tableDelta = ((float32(framesPerBuffer) * freqency) / float32(samplingRate))
  let table = makeTable(framesPerBuffer, square)

  type
    TState = tuple[n: float32]
    TStereo = tuple[left, right: float32]

  proc mean_interpolate(x: float32): float32 =
    let
      xprev = int(m.floor(x)) mod table.len
      xnext = int(m.ceil(x)) mod table.len
    return table[xprev] + table[xnext] / 2

  proc linear_interpolate(x: float32): float32 =
    let
      ratio = x - m.floor(x)
      xprev = table[int(m.floor(x)) mod table.len]
      xnext = table[int(m.ceil(x)) mod table.len]
    return xprev * ratio + xnext * (1 - ratio)

  proc fillingWithTable(inBuf, outBuf: pointer,
                        framesPerBuf: culong,
                        timeInfo: ptr TStreamCallbackTimeInfo,
                        stateusFlags: TStreamCallbackFlags,
                        userData: pointer): cint {.cdecl.} =
    var
      outBuf = cast[ptr array[framesPerBuffer, TStereo]](outBuf)
      state = cast[ptr TState](userData)

    proc cropPos(x: float32): float32 =
      if x > float32(table.len):
        result = x - float32(table.len)
      else:
        result = x

    for i in 0..<int(framesPerBuf):
      let tablePos = state.n + float32(i) * tableDelta
      let val = 0.3'f32 * linear_interpolate(cropPos(tablePos))
      outBuf[i] = (val, val)
    state.n = cropPos(state.n + float32(framesPerBuf) * tableDelta)

  var
    stream: PStream
    state = (n: 0)

  discard PA.OpenDefaultStream(
    cast[PStream](stream.addr),
    numInputChannels = 0,
    numOutputChannels = 2,
    sampleFormat = sfFloat32,
    sampleRate = samplingRate,
    framesPerBuffer = framesPerBuffer,
    streamCallback = fillingWithTable,
    userData = cast[pointer](state.addr))

  discard PA.StartStream(stream)
  PA.Sleep(2000)
  discard PA.StopStream(stream)
  discard PA.CloseStream(stream)

  echo "============== terminate pa   ==============="
  echo repr(PA.Terminate())
