from os import commandLineParams
import math as m

import vorbis/vorbisfile as VF
import portaudio as PA

import primitivepkg.oscillators.wave_table as wt
import primitivepkg.utils

type
  SoundOut* = ref object
    channelNum*: int
    sampleFormat*: PA.TSampleFormat
    sampleRate*: float64
    bufferSize*: uint64

const seq_step = 16
type
  ASDR* = enum
    None, Attack, Decay, Sustin, Release
  StepSequencer* = ref object
    tempo*: float64
    sequence*: string
    noteDuration*: int
    osc*: wt.WaveTableOcillator

    beat*: float64
    time*: float64
    state*: ASDR
    envelope*: float32

type
  TSound* = tuple[sndout: SoundOut, seq: StepSequencer]
  TStereo* = tuple[left, right: float32]

proc playWithPA(s: string) =
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
      freq = 440
      tableDelta = (float32(osc.tableSize) * float32(freq)) / snd.sndout.sampleRate
      timeDelta = 1 / snd.sndout.sampleRate

    echo $(snd.seq.state) & ". " & $(snd.seq.time) & ", " & $(snd.seq.beat)
    for i in 0..<int(snd.sndout.bufferSize):
      let val = snd.seq.envelope * osc.interpolFn(
        crop(osc.tablePos, float32(osc.tableSize)), osc)
      outBuf[i] = (val, val)
      osc.tablePos = osc.tablePos + tableDelta

      let before_beat = snd.seq.beat
      snd.seq.time = snd.seq.time + timeDelta
      snd.seq.beat = snd.seq.beat + timeDelta * snd.seq.tempo / 60

      const
        attack = 0.01
        decay = 0.1
        sustin = 0.2
        release = 0.4
      if m.floor(snd.seq.beat) - m.floor(before_beat) == 1:
        snd.seq.state = ASDR.Attack
        snd.seq.envelope = (snd.seq.beat - m.floor(snd.seq.beat)) / attack
      elif snd.seq.beat - m.floor(snd.seq.beat) < attack:
        snd.seq.state = ASDR.Attack
        snd.seq.envelope = (snd.seq.beat - m.floor(snd.seq.beat)) / attack
      elif snd.seq.beat - m.floor(snd.seq.beat) < attack + decay:
        snd.seq.state = ASDR.Decay
        snd.seq.envelope = 1 - (snd.seq.beat - m.floor(snd.seq.beat) - attack) / decay  + sustin
      elif snd.seq.beat - m.floor(snd.seq.beat) < attack + decay + sustin:
        snd.seq.state = ASDR.Sustin
        snd.seq.envelope = sustin
      elif snd.seq.beat - m.floor(snd.seq.beat) < attack + decay + sustin + release:
        snd.seq.state = ASDR.Release
        snd.seq.envelope = 0
      elif snd.seq.beat - m.floor(snd.seq.beat) > attack + decay + sustin + release:
        snd.seq.state = ASDR.None
        snd.seq.envelope = 0

  var
    stream: PStream
    sndout = SoundOut(
      channelNum: 2,
      sampleFormat: PA.TSampleFormat.sfFloat32,
      sampleRate: 44100,
      bufferSize: 1024)
    osc = wt.WaveTableOcillator(
      tableSize: 512, interpolFn: wt.linear_interpolate, tablePos: 0, volume: 0.5)
    stepseq = StepSequencer(
      tempo: 120,
      sequence: s,
      noteDuration: 100,
      osc: osc,
      time: 0,
      beat: 0,
      state: ASDR.None,
      envelope: 0)
    snd: TSound = (sndout, stepseq)

  osc.waveTable = wt.makeTable(osc, wt.tri)


  discard PA.OpenDefaultStream(
    cast[PStream](stream.addr),
    numInputChannels = 0,
    numOutputChannels = cint(snd.sndout.channelNum),
    sampleFormat = snd.sndout.sampleFormat,
    sampleRate = cdouble(snd.sndout.sampleRate),
    framesPerBuffer = culong(snd.sndout.bufferSize),
    streamCallback = fillingWithTable,
    userData = cast[pointer](snd.addr))

  discard PA.StartStream(stream)
  PA.Sleep(4000)
  discard PA.StopStream(stream)
  discard PA.CloseStream(stream)

  echo "============== terminate pa   ==============="
  echo repr(PA.Terminate())


when isMainModule:
  let
    args = commandLineParams()

  if args.len == 0:
    playWithPA("0000000000000000")

  elif args.len == 1:
    playWithPA(args[0])

  # var vf: VF.TOggVorbis_File

  # if VF.fopen(args[0], vf.addr) == 0:
  #   echo "cannot open '" & args[0] & "'"
  #   quit(1)

  # echo "============== show .ogg info ==============="
  # echo "filename: '" & args[0] & "'"
  # echo repr(VF.info(vf.addr, -1))
