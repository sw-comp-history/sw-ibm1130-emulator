; Demo: arithmetic
;
; Computes (5 + 3) * 4 and stores the result. Demonstrates LD,
; A (add), M (multiply -> ACC,EXT pair) and STO. After execution
; the word at RESULT should hold 32 (decimal).
;
; Memory layout:
;   word 0..N    code
;   FIVE         literal 5
;   THREE        literal 3
;   FOUR         literal 4
;   RESULT       output (computed)

        LD   L FIVE
        A    L THREE       ; ACC = 8
        M    L FOUR        ; ACC,EXT = 32 (low half in EXT after multiply)
        STO  L RESULT      ; store ACC (high half) to RESULT
        WAIT

FIVE:   DC   5
THREE:  DC   3
FOUR:   DC   4
RESULT: DC   0
