(bpm 120)
(measure 4 4)

(def $hat-pat (pat (r 2) (k 3) (k 3) (k 3) (k 3)
                   loop))
(def $hat-osc (rand 0))
(def $hat-eg (adsr 0 0.05 0 0))

(def $kick-pat (pat (k 3) (k 3) (k 3) (k 3)
                    loop))
(def $kick-fmod (adsr 0 0.1 0 0))
(def $kick-osc (sine 0 (* 300 $kick-fmod)))
(def $kick-eg (adsr 0 0.2 0 0))

(def $synth-pat (pat (c4 2) (d+4 1) (d+4 1) (d+4 2) (c4 2)
                     (c4 2) (c4 2) (c4 2) (c4 2)
                     loop))
(def $synth-osc (saw 0 0))
(def $synth-eg (adsr 0 0.3 0.3 0))
(def $synth-lpfmod (+ 700 (* 300 (tri 0 1))))

(def $bass-pat (pat (r 2) (c2 2) (r 2) (c2 2)
                    (r 2) (c2 2) (r 2) (c2 2)
                    (r 2) (c2 2) (r 2) (c2 2)
                    (r 2) (d2 2) (r 2) (d2 2)
                    loop))
(def $bass-osc (saw 0 0))
(def $bass-eg (adsr 0.05 0.3 0 0))

(out 0.3
     (seq $hat-pat $hat-osc $hat-eg)
     (seq $kick-pat $kick-osc (trig $kick-eg $kick-fmod))
     (gain 0.4 (lpf $synth-lpfmod 20 (seq $synth-pat $synth-osc $synth-eg)))
     (lpf (+ 600 (* 200 (tri 0 1)) (* 100 $bass-eg)) 20
          (seq $bass-pat $bass-osc $bass-eg)))
