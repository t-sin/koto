from os import commandLineParams
import math as m

import vorbis/vorbisfile as VF
import portaudio as PA

import primitivepkg.oscillators.wave_table as wt
import primitivepkg.envelope as eg
import primitivepkg.utils

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

    beat*: float64
    time*: float64
    env*: eg.Envelope

type
  TStereo* = tuple[left, right: float32]

type
  TSoundRef = ref object
    sndout: SoundOut
    seq: StepSequencer

proc oscillateStepSeq(snd: TSoundRef): float32 =
  let
    freq = 440'f
    osc = snd.seq.osc

  #echo $(snd.seq.env.state) & ". " & $(snd.seq.time) & ", " & $(snd.seq.beat)
  let
    oscVal = oscillate(osc, freq, snd.sndout.sampleRate)
    envelope = generateEnvelope(snd.seq.env, snd.seq.time)

  return oscVal * envelope

proc processTime(snd: TSoundRef): float64 =
  let
    timeDelta = 1 / snd.sndout.sampleRate
    before_beat = snd.seq.beat

  snd.seq.time = snd.seq.time + timeDelta
  snd.seq.beat = snd.seq.beat + timeDelta * snd.seq.tempo / 60

  return snd.seq.beat

proc processPaBuffer(inBuf, outBuf: pointer,
                     framesPerBuf: culong,
                     timeInfo: ptr TStreamCallbackTimeInfo,
                     stateusFlags: TStreamCallbackFlags,
                     userData: pointer): cint {.cdecl.} =
  let
    outBuf = cast[ptr array[int, TStereo]](outBuf)
    snd = cast[TSoundRef](userData)

  for i in 0..<int(snd.sndout.bufferSize):
    let val = oscillateStepSeq(snd)
    outBuf[i] = (val, val)

    let
      before_beat = snd.seq.beat
      beat = processTime(snd)

    # note on
    # TODO: trigger from another thread
    if m.floor(snd.seq.beat) - m.floor(before_beat) == 1:
      eg.noteOn(snd.seq.env, snd.seq.time)
    elif snd.seq.beat - m.floor(snd.seq.beat) > 0.7:
      eg.noteOff(snd.seq.env, snd.seq.time)


proc playWithPA(snd: TSoundRef) =
  echo "============== initialize pa  ==============="
  echo repr(PA.Initialize())

  var
    stream: PStream

  discard PA.OpenDefaultStream(
    cast[PStream](stream.addr),
    numInputChannels = 0,
    numOutputChannels = cint(snd.sndout.channelNum),
    sampleFormat = snd.sndout.sampleFormat,
    sampleRate = cdouble(snd.sndout.sampleRate),
    framesPerBuffer = culong(snd.sndout.bufferSize),
    streamCallback = processPaBuffer,
    userData = cast[pointer](snd))

  type KeyboardInterruptError = object of Exception
  proc handleError() {.noconv.} =
    echo "Keyboard Interrupt"
    raise newException(KeyboardInterruptError, "Keyboard Interrupt")

  setControlCHook(handleError)

  discard PA.StartStream(stream)
  try:
    while true:
      PA.Sleep(1)

  except KeyboardInterruptError:
    discard PA.StopStream(stream)
    discard PA.CloseStream(stream)
    echo repr(PA.Terminate())
    echo "============== terminate pa   ==============="
    quit 0


when isMainModule:
  let
    args = commandLineParams()

  var
    sndout = SoundOut(
      channelNum: 2,
      sampleFormat: PA.TSampleFormat.sfFloat32,
      sampleRate: 44100,
      bufferSize: 1024)
    osc = wt.WaveTableOcillator(
      interpolFn: wt.linear_interpolate, tablePos: 0, volume: 0.5)
    env = Envelope(
      a: 0.1,
      d: 0.1,
      s: 0.5,
      r: 0.1,
      state: ASDR.None)
    stepseq = StepSequencer(
      tempo: 120,
      sequence: "0000000000000000",
      noteDuration: 100,
      osc: osc,
      time: 0,
      beat: 0,
      env: env)
    snd = TSoundRef(
      sndout: sndout,
      seq: stepseq)

  osc.waveTable = wt.makeTable(osc, 256, wt.saw)

  playWithPA(snd)

  # var vf: VF.TOggVorbis_File

  # if VF.fopen(args[0], vf.addr) == 0:
  #   echo "cannot open '" & args[0] & "'"
  #   quit(1)

  # echo "============== show .ogg info ==============="
  # echo "filename: '" & args[0] & "'"
  # echo repr(VF.info(vf.addr, -1))
