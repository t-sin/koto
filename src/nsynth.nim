from os import commandLineParams
import math as m

import vorbis/vorbisfile as VF
import portaudio as PA

import oscillators.wave_table as wt


proc playWithPA() =
  echo "============== initialize pa  ==============="
  echo repr(PA.Initialize())

  var wt_osc = wt.WaveTableOcillator(
    tableSize: 1024, interpolFn: wt.linear_interpolate, tablePos: 0)
  wt_osc.waveTable = wt.makeTable(wt_osc, wt.square)

  const
    framesPerBuffer = 1024
    samplingRate = 44_100
    freqency = 880

  type
    TStereo = tuple[left, right: float32]

  proc fillingWithTable(inBuf, outBuf: pointer,
                        framesPerBuf: culong,
                        timeInfo: ptr TStreamCallbackTimeInfo,
                        stateusFlags: TStreamCallbackFlags,
                        userData: pointer): cint {.cdecl.} =
    var outBuf = cast[ptr array[int, TStereo]](outBuf)
    var wtdata = cast[ptr wt.WaveTableOcillator](userData)
    wt.fillBuffer(wtdata[], freqency, samplingRate, outBuf, int(framesPerBuf))

  var stream: PStream

  discard PA.OpenDefaultStream(
    cast[PStream](stream.addr),
    numInputChannels = 0,
    numOutputChannels = 2,
    sampleFormat = sfFloat32,
    sampleRate = samplingRate,
    framesPerBuffer = framesPerBuffer,
    streamCallback = fillingWithTable,
    userData = cast[pointer](wt_osc.addr))

  discard PA.StartStream(stream)
  PA.Sleep(2000)
  discard PA.StopStream(stream)
  discard PA.CloseStream(stream)

  echo "============== terminate pa   ==============="
  echo repr(PA.Terminate())


let args = commandLineParams()

if args.len == 0:
  playWithPA()

elif args.len == 1:
  var vf: VF.TOggVorbis_File

  if VF.fopen(args[0], vf.addr) == 0:
    echo "cannot open '" & args[0] & "'"
    quit(1)

  echo "============== show .ogg info ==============="
  echo "filename: '" & args[0] & "'"
  echo repr(VF.info(vf.addr, -1))
