;; from Fancy Free by Donald Byrd
(bpm 140)
(measure 4 4)

;; hihat
(def $hihat-pat (pat (k 1) (r 1) (k 1) (k 1) (k 1) (r 1) (k 1) (k 1)
                     (k 1) (r 1) (k 1) (k 1) (k 1) (r 1) (k 1) (k 1)
                     loop))
(def $hihat-osc (rand 0))
(def $hihat-eg (adsr 0 0.025 0.05 0))


;; snare
(def $snare-pat (pat (r 3) (k 3) (r 2) (k 2) (r 2) (r 3)
                     (k 2) (k 2) (r 3) (k 2) (r 3)
                     (r 3) (k 3) (r 2) (k 2) (r 2) (r 3)
                     (k 2) (k 2) (r 3) (k 2) (k 2) (r 2)
                     loop))
(def $snare-osc (rand 10))
(def $snare-eg (adsr 0 0.3 0 0))

;; kick
(def $kick-pat (pat (a1 3) (a1 3) (r 2) (a1 3) (a1 3) (r 2)
                    (a1 3) (a1 2) (r 2) (a1 2) (a1 2)
                    loop))
(def $kick-fmod (adsr 0 0.1 0 0))
(def $kick-osc (sine 0 0))
(def $kick-eg (adsr 0 0.2 0 0))

;; bassline
(def $bass-pat (pat (g2 1) (r 1) (r 2) (g2 3) (r 2) (g2 2) (r 2) (g2 2) (r 2)
                    (g2 2) (e2 2) (e2 2) (e+2 2) (e+2 2) (f+2 2) (f+2 2)
                    loop))
(def $bass-osc (saw 0 0))
(def $bass-eg (adsr 0.05 0.3 0 0))

(def $key1-pat (pat (c5 1) (r 1) (r 2) (c5 3) (r 2) (c5 1) (r 1) (r 2)
                    (b4 3) (b4 2) (r 2) (c5 1) (r 1) (b4 1) (r 1) (r 2) (r 3)
                    loop))
(def $key1-osc (sine 0 0))
(def $key1-eg (adsr 0.08 0.12 0.7 0))

(def $key2-pat (pat (e5 1) (r 1) (r 2) (e5 3) (r 2) (e5 1) (r 1) (r 2)
                    (e5 3) (e5 2) (r 2) (e5 1) (r 1) (d5 1) (r 1) (r 2) (r 3)
                    loop))
(def $key2-osc (sine 0 0))
(def $key2-eg (adsr 0.08 0.12 0.7 0))

(def $key3-pat (pat (g5 1) (r 1) (r 2) (g5 3) (r 2) (g5 1) (r 1) (r 2)
                    (g5 3) (g5 2) (r 2) (g5 1) (r 1) (g5 1) (r 1) (r 2) (r 3)
                    loop))
(def $key3-osc (sine 0 0))
(def $key3-eg (adsr 0.08 0.12 0.7 0))

(out 0.3
     (* 0.7 (seq $hihat-pat $hihat-osc 0 $hihat-eg))
     (* 0.5 (seq $snare-pat $snare-osc 0 $snare-eg))
     (* 0.6 (seq $kick-pat $kick-osc (* 300 $kick-fmod) (trig $kick-eg $kick-fmod)))
     (lpf (+ 600 (* 200 (tri 0 1)) (* 100 $bass-eg)) 10
          (seq $bass-pat $bass-osc 0 $bass-eg))
     (* 0.8 (delay 0.1 0.3 0.3
                   (+ (seq $key1-pat $key1-osc 0 $key1-eg)
                      (seq $key2-pat $key2-osc 0 $key2-eg)
                      (seq $key3-pat $key3-osc 0 $key3-eg)))))
