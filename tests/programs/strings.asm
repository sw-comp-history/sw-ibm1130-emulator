; Demo: copy a "string" word-by-word until a sentinel
;
; Copies words from SRC to DST until the sentinel 0xFFFF. Uses
; LDX, indexed LD/STO via XR1, MDX to increment XR1, and a direct
; masked BSC long form: `BSC L END, 0x20` = branch if Z (ACC == 0,
; i.e. sentinel matched after subtracting it).
;
; Mask bit 0x20 = Z (ACC == 0) per Moore's 1968 :EQUAL constant.
;
; The values 0x0048/0x0049/0x004A happen to spell "HIJ" if
; interpreted as ASCII; this demo treats them as opaque words.
; Encoding-pipeline plans live in
; gen-isa/docs/character-encoding-plan.md.

        LD   L ZERO
        STO  L LEN           ; LEN = 0
        LDX  1, 0            ; XR1 = 0 (short-form immediate)

LOOP:   LD   L 1, SRC        ; ACC = SRC[XR1]
        S    L SENTINEL      ; ACC = SRC[XR1] - 0xFFFF
        BSC  L END, 0x20     ; branch to END if Z (sentinel matched)
        LD   L 1, SRC        ; reload (S clobbered ACC)
        STO  L 1, DST        ; DST[XR1] = ACC
        LD   L LEN
        A    L ONE
        STO  L LEN           ; LEN += 1
        MDX  1, 1            ; XR1 += 1
        BSC  L LOOP, 0       ; unconditional jump back

END:    WAIT

ZERO:     DC 0
ONE:      DC 1
SENTINEL: DC 0xFFFF
LEN:      DC 0
SRC:      DC 0x0048
          DC 0x0049
          DC 0x004A
          DC 0xFFFF
DST:      DC 0
          DC 0
          DC 0
          DC 0
