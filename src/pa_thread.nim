import math as m

import portaudio as PA

import kotopkg.oscillators.wave_table as wt
import kotopkg.envelope as eg
import snd_types


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


proc playWithPA*(snd: TSoundRef) {.thread.} =
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
