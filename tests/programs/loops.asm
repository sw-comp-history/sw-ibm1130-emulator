; Demo: counted loop
;
; Computes 1+2+...+10 = 55 by iterating. Uses a direct masked
; BSC long form: `BSC L LOOP, 0x18` branches back to LOOP if ACC is
; positive (0x08) or negative (0x10) -- i.e. non-zero. When ACC
; reaches 0, fall through to store RESULT and halt.
;
; Mask bit assignments (per Moore's 1968 :EVEN/:POSITIVE/:NEGATIVE/
; :EQUAL constants): 0x04=E, 0x08=+, 0x10=-, 0x20=Z, 0x40=C.

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
        BSC  L LOOP, 0x18    ; branch if + or - (ACC != 0)
        LD   L SUM
        STO  L RESULT
        WAIT

ZERO:   DC   0
ONE:    DC   1
TEN:    DC   10
I:      DC   0
SUM:    DC   0
RESULT: DC   0
