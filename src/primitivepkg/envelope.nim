type
  ASDR* = enum
    None, Attack, Decay, Sustin, Release
  Envelope* = ref object
    a*: float32
    d*: float32
    s*: float32
    r*: float32

    state*: ASDR
    startTime*: float32

proc noteOn*(env: Envelope, startTime: float32) =
  env.startTime = startTime
  env.state = ASDR.Attack

proc generateEnvelope*(env: Envelope, time: float32): float32 =
  let noteTime = time - env.startTime

  if env.state in [ASDR.None, ASDR.Attack] and noteTime < env.a:
    env.state = ASDR.Attack
    return noteTime / env.a

  elif env.state in [ASDR.Attack, ASDR.Decay] and noteTime < env.a + env.d:
    env.state = ASDR.Decay
    return 1 - (noteTime - env.a) / env.d + env.s

  elif env.state in [ASDR.Decay, ASDR.Sustin] and noteTime < env.a + env.d + env.s:
    env.state = ASDR.Sustin
    return env.s

  elif env.state in [ASDR.Sustin, ASDR.Release] and noteTime < env.a + env.d + 0.1 + env.r:
    env.state = ASDR.Release
    return (noteTime - env.a- env.d - env.s) / env.r * env.s

  elif noteTime > env.a + env.d + env.s + env.r:
    env.state = ASDR.None
    return 0

