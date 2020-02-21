(bpm 120)
(measure 4 4)

;; hi hat
(def $hat-pat (pat (r 2) (k 3) (k 3) (k 3) (k 3)
                   loop))
(def $hat-osc (rand 0))
(def $hat-eg (adsr 0 0.05 0 0))

;; kick
(def $kick-pat (pat (a2 3) (a2 3) (a2 3) (a2 3)
                    loop))
(def $kick-fmod (adsr 0 0.1 0 0))
(def $kick-osc (sine 0 0))
(def $kick-eg (adsr 0 0.2 0 0))

;; synth1
(def $synth-pat (pat (a+4 1) (a+4 1) (a+4 1) (a+4 1) (a+4 1) (a+4 1) (a+4 1) (a+4 1)
                     (a+4 1) (a+4 1) (a+4 1) (a+4 1) (a+4 1) (a+4 1) (a+4 1) (a+4 1)
                     loop))
(def $synth-osc (saw 0 0))
(def $synth-eg (adsr 0 (* 0.1 (+ 1.0 (sine 0 0.4))) 0.0 0))
(def $synth-lpfmod (+ 700 (* 300 (tri 0 1))))

;; bassline
(def $bass-pat (pat (r 2) (d+2 2) (r 2) (d+2 2) (r 2) (d+2 2) (r 2) (d+2 2)
                    (r 2) (d+2 2) (r 2) (d+2 2) (r 2) (d+2 2) (r 2) (d+2 2)
                    (r 2) (d+2 2) (r 2) (d+2 2) (r 2) (d+2 2) (r 2) (d+2 2)
                    (r 2) (g+2 2) (r 2) (g+2 2) (r 2) (a+2 2) (r 2) (a+2 2)
                    loop))
(def $bass-osc (saw 0 0))
(def $bass-eg (adsr 0.05 0.3 0 0))

;; synth2
(def $synth2-pat (pat (r 2) (g+5 1) (r 1) (r 2) (g+5 1) (r 1) (r 2)
                      (g4 2) (r 2) (g4 2) (r 2)
                      loop))
(def $synth2-osc (wavetable (pulse 0 1 0.25) (phase (saw 0 440))))
(def $synth2-osc2 (saw 0 0))
(def $synth2-eg (adsr 0.05 0 1 0.15))

(out 0.3
     (seq $hat-pat $hat-osc 0 $hat-eg)
     ;; frequency modulation with envelove generator
     (seq $kick-pat $kick-osc (* 300 $kick-fmod) (trig $kick-eg $kick-fmod))
     (gain 0.4 (delay 0.25 0.3 0.5 (lpf $synth-lpfmod 20 (seq $synth-pat $synth-osc 0 $synth-eg))))
     ;; cutoff frequency modulated bass
     (lpf (+ 600 (* 200 (tri 0 1)) (* 100 $bass-eg)) 20
          (seq $bass-pat $bass-osc 0 $bass-eg))
     ;; supersaw with toremolo and auto panning
     (pan (sine 0 0.5)
          (gain 0.5
                (* (+ 0.7 (* 0.3 (sine 0 40)))
                   (+ (seq $synth2-pat $synth2-osc 0 $synth2-eg)
                      (seq $synth2-pat $synth2-osc2 2 $synth2-eg))))))
