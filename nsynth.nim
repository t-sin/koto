from os import commandLineParams
import math as m

import vorbis/vorbisfile as VF
import portaudio as PA

import oscillators.wave_table as wt


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

  let wt_osc = wt.WaveTableOcillator(tableSize: 512, interpolType: wt.linear_interpolate)
  wt_osc.waveTable = wt.makeTablef(wt_osc.tableSize, wt.square)

  const framesPerBuffer = 1024
  const samplingRate = 44_100
  const freqency = 440
  const tableDelta = ((float32(wt.tableSize) * freqency) / float32(samplingRate))

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

    proc cropPos(x: float32): float32 =
      if x > float32(wt.tableSize):
        result = x - float32(wt.tableSize)
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
