import portaudio as PA

import oscillators.wave_table as wt
import envelope as eg


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
  TSoundRef* = ref object
    sndout*: SoundOut
    seq*: StepSequencer
