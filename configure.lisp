(bpm 120)
(measure 4 4)

(def $pat1 (pat (c4 1) (d 1) (e 1) (f 1) (g 1) (a 1) (b 1) (c5 1) (r 4)
                 loop))

(def $osc1 (wavetable (pulse 0 1 0.5)(phase (saw 0 440))))
(def $eg1 (adsr 0 (gain 0.2 (offset 1 (saw 0 0.25))) 0.0 0.1))

(def $pat2 (pat (c 2) (r 2) (c 2) (r 2) (c 2) (r 2) (c 2) (r 2) loop))
(def $osc2 (rand 0))
(def $eg2 (adsr 0 0.1 0.005 0))

(def $pat3 (pat (c3 2) (c3 2) (r 1) (c3 2) (c3 2) (c3 2) (r 1) (a+5 1) (r 1) (f5 1) (r 1) loop))
(def $osc3 (saw 0 440))
(def $eg3 (adsr 0.02 0.15 0.6 0))
(def $feg3 (adsr 0 0.1 0.3 0))

(out 0.3
  (pan 0.2 (gain 0.15 (delay 0.3 0.5 1 (seq $pat1 $osc1 $eg1))))
  (gain 0.25 (seq $pat2 $osc2 $eg2))
  (gain 0.2 (lpf (+ 500 (+ 200 (* 200 (tri 0 0.5))) (* 1200 $feg3)) 10 (seq $pat3 $osc3 (trig $eg3 $feg3)))))
