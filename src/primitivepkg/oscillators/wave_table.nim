import math as m

import ../utils

type
  InterpolationType* = enum
    MEAN, LINEAR

  WaveTableOcillator* = ref object
    waveTable*: seq[float32]
    interpolFn*: proc (x: float32, t: seq[float32]): float32
    tablePos*: float32
    volume*: float32


proc makeTable*(wt: WaveTableOcillator,
                size: int,
                fn: proc (n: int, max: int): float32): seq[float32] =
  result = newSeq[float32](size)
  for i in 0..<size:
    result[i] = fn(i, size)


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

# table interploation
proc mean_interpolate*(x: float32, table: seq[float32]): float32 =
  let
    xprev = int(m.floor(x)) mod table.len
    xnext = int(m.ceil(x)) mod table.len
  return table[xprev] + table[xnext] / 2

proc linear_interpolate*(x: float32, table: seq[float32]): float32 =
  let
    ratio = x - m.floor(x)
    xprev = table[int(m.floor(x)) mod table.len]
    xnext = table[int(m.ceil(x)) mod table.len]
  return xprev * ratio + xnext * (1 - ratio)
