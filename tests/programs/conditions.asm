; Demo: conditional branching (max of two values)
;
; Computes max(A, B) and stores it. Demonstrates the 1130 idiom
; for conditional branches: BSC short (skip-on-cond) followed by
; BSC L unconditional. Inputs A=12, B=17; expected RESULT=17.
;
; Strategy:
;   ACC <- A - B
;   if ACC < 0 (A < B): branch to STORE_B
;     == "skip-if-NOT-negative; jump-to-STORE_B"
;     mask 0x3D = Z|+|E|C|O (everything except '-')
;   STORE_A: RESULT <- A
;   END: WAIT
;
; STORE_B: RESULT <- B; jump to END

        LD   L A
        S    L B             ; ACC = A - B
        BSC  0x05            ; skip next if ACC == 0 or ACC > 0 (A >= B)
        BSC  L STORE_B, 0    ; A < B -> branch
        LD   L A             ; A >= B -> result = A
        BSC  L END, 0        ; jump to END
STORE_B:
        LD   L B             ; result = B
END:    STO  L RESULT
        WAIT

A:      DC   12
B:      DC   17
RESULT: DC   0
