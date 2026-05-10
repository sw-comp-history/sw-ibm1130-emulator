; Demo: counted loop
;
; Computes 1+2+...+10 = 55 by iterating. Demonstrates LD, STO, A,
; S, BSC short (skip-on-condition), and BSC L unconditional jump.
;
; Conditional branching idiom on the 1130 has TWO instructions:
;   BSC mask           ; skip next instr if condition matches
;   BSC L target, 0    ; unconditional jump (mask=0)
; Combined: "branch unless condition matches" (== conditional jump).
;
; Strategy:
;   SUM <- 0;  I <- 10
;   loop:
;     SUM <- SUM + I
;     I <- I - 1
;     if I == 0 break              ; encoded as: skip-if-NZ; jump
;     goto loop
;   RESULT <- SUM

        LD   L ZERO
        STO  L SUM           ; SUM = 0
        LD   L TEN
        STO  L I             ; I = 10
LOOP:   LD   L SUM
        A    L I
        STO  L SUM           ; SUM <- SUM + I
        LD   L I
        S    L ONE
        STO  L I             ; I <- I - 1
        BSC  0x01            ; skip next if ACC == 0 (loop done)
        BSC  L LOOP, 0       ; otherwise jump back
        LD   L SUM
        STO  L RESULT
        WAIT

ZERO:   DC   0
ONE:    DC   1
TEN:    DC   10
I:      DC   0
SUM:    DC   0
RESULT: DC   0
