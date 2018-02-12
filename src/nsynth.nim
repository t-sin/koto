from os import commandLineParams
import math as m

import vorbis/vorbisfile as VF
import portaudio as PA

import oscillators.wave_table as wt


type
  SoundOut* = ref object
    channelNum*: int
    sampleFormat*: PA.TSampleFormat
    sampleRate*: float64
    bufferSize*: uint64

const seq_step = 16
type
  StepSequencer* = ref object
    tempo*: float64
    sequence*: string
    noteDuration*: int
    osc*: wt.WaveTableOcillator
    time*: float64

type
  TSound* = tuple[sndout: SoundOut, seq: StepSequencer]
  TStereo* = tuple[left, right: float32]

proc playWithPA() =
  echo "============== initialize pa  ==============="
  echo repr(PA.Initialize())

  proc fillingWithTable(inBuf, outBuf: pointer,
                        framesPerBuf: culong,
                        timeInfo: ptr TStreamCallbackTimeInfo,
                        stateusFlags: TStreamCallbackFlags,
                        userData: pointer): cint {.cdecl.} =
    let
      outBuf = cast[ptr array[int, TStereo]](outBuf)
      snd = cast[ptr TSound](userData)
      osc = snd.seq.osc
    wt.fillBuffer(osc, 880, snd.sndout.sampleRate, outBuf, int(snd.sndout.bufferSize))

  var
    stream: PStream
    sndout = SoundOut(
      channelNum: 2,
      sampleFormat: PA.TSampleFormat.sfFloat32,
      sampleRate: 44100,
      bufferSize: 1024)
    osc = wt.WaveTableOcillator(
      tableSize: 512, interpolFn: wt.linear_interpolate, tablePos: 0)
    stepseq = StepSequencer(
      tempo: 120,
      sequence: "0000000000000000",
      noteDuration: 100,
      osc: osc,
      time: 0)
    snd = (sndout, stepseq)

  osc.waveTable = wt.makeTable(osc, wt.tri)

  discard PA.OpenDefaultStream(
    cast[PStream](stream.addr),
    numInputChannels = 0,
    numOutputChannels = cint(sndout.channelNum),
    sampleFormat = sndout.sampleFormat,
    sampleRate = cdouble(sndout.sampleRate),
    framesPerBuffer = culong(sndout.bufferSize),
    streamCallback = fillingWithTable,
    userData = cast[pointer](snd.addr))

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
