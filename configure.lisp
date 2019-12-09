(bpm 120)
(measure 4 4)

(def $mod1 (+ 440 (* 2 (sine 0 10))))
(def $osc1 (saw 0 $mod1))
(def $osc2 (saw 0 (+ 10 $mod1)))

(out 0.8 $osc1 $osc2)
