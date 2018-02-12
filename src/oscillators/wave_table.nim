import math as m


type
  InterpolationType* = enum
    MEAN, LINEAR

  WaveTableOcillator* = ref object
    tableSize*: int
    waveTable*: seq[float32]
    interpolType*: proc (x: float32, wt: WaveTableOcillator): float32


proc makeTable*(wt: WaveTableOcillator,
                fn: proc (n: int, max: int): float32): seq[float32] =
  result = newSeq[float32](wt.tableSize)
  for i in 0..<wt.tableSize:
    result[i] = fn(i, wt.tableSize)

# wave form generator
# these can be more gereral...?
proc sin*(n: int, len: int): float32 =
  let i = float(n) / float(len)
  result = m.sin(i * 2 * m.PI)

proc saw*(n: int, len: int): float32 =
  result = float32(n) / float32(len)

proc tri*(n: int, len: int): float32 =
  let
    hperiod = int(len / 2)
    qperiod = int(len / 4)

  if n < qperiod:
    result = (float32(n) / float32(qperiod))
  elif n < (hperiod + qperiod):
    result = -(float32(n) / float32(qperiod)) + 2'f32
  else:
    result = float32(n) / float32(qperiod) - 4'f32

proc square*(n: int, len: int): float32 =
  let
    hperiod = int(len / 2)
    qperiod = int(len / 4)

  if n < qperiod:
    result = 1'f32
  elif n < (hperiod + qperiod):
    result = 0'f32
  else:
    result = 1'f32

proc mean_interpolate*(x: float32, wt: WaveTableOcillator): float32 =
  let
    xprev = int(m.floor(x)) mod wt.tableSize
    xnext = int(m.ceil(x)) mod wt.tableSize
  return wt.waveTable[xprev] + wt.waveTable[xnext] / 2

proc linear_interpolate*(x: float32, wt: WaveTableOcillator): float32 =
  let
    ratio = x - m.floor(x)
    xprev = wt.waveTable[int(m.floor(x)) mod wt.tableSize]
    xnext = wt.waveTable[int(m.ceil(x)) mod wt.tableSize]
  return xprev * ratio + xnext * (1 - ratio)
