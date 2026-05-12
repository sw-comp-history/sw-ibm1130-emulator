; Demo: conditional branching (max of two values)
;
; Computes max(A, B) using a direct masked BSC long form.
; With A=12 and B=17, RESULT should be 17.
;
; The 1130 conditional-branch encoding (per Moore's 1968 listing):
;   0x04 = Even, 0x08 = Positive, 0x10 = Negative, 0x20 = Zero,
;   0x40 = Carry.
; BSC L target, mask  branches if mask == 0 OR any masked
; condition holds. So `BSC L STORE_B, 0x10` = branch if ACC < 0.

        LD   L A
        S    L B             ; ACC = A - B
        BSC  L STORE_B, 0x10 ; branch if N (ACC < 0, i.e. A < B)
        LD   L A             ; A >= B -> result = A
        BSC  L END, 0        ; unconditional jump
STORE_B:
        LD   L B             ; result = B
END:    STO  L RESULT
        WAIT

A:      DC   12
B:      DC   17
RESULT: DC   0
