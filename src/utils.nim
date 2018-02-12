
proc crop*(a: float32, max: float32): float32 =
  if a >= float32(max):
    result = a - float32(max)
  else:
    result = a
