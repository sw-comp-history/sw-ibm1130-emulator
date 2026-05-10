; Demo: copy a "string" word-by-word until a sentinel
;
; The 1130 has no character type; "strings" are sequences of
; words. This demo copies words from SRC to DST until it sees
; the sentinel 0xFFFF, demonstrating LDX, indexed LD/STO via XR1,
; and the standard skip+jump conditional-branch idiom.
;
; SRC contents are three arbitrary 16-bit words plus a sentinel.
; The values 0x0048/0x0049/0x004A happen to spell "HIJ" in ASCII,
; but this demo treats them as opaque words. Historically, IBM
; 1130 software running alongside System/360 hosts stored text in
; EBCDIC, packed two bytes per word; 1130 device I/O used per-
; device codes (Hollerith for cards, PTTC for paper tape, printer-
; specific codes). We will add explicit encoding support in a
; future saga -- see gen-isa/docs/character-encoding-plan.md.
;
; After: DST holds 0x0048 0x0049 0x004A; LEN = 3.

        LD   L ZERO
        STO  L LEN           ; LEN = 0
        LDX  L 1, ZERO       ; XR1 = 0 (index into SRC/DST)

LOOP:   LD   L 1, SRC        ; ACC = SRC[XR1]   (long form: absolute base)
        S    L SENTINEL      ; ACC = SRC[XR1] - 0xFFFF
        BSC  0x06            ; skip next if ACC NOT 0 (any of -/+ matches)
        BSC  L END, 0        ; sentinel matched -> jump to END
        LD   L 1, SRC        ; reload (S clobbered ACC)
        STO  L 1, DST        ; DST[XR1] = ACC
        LD   L LEN
        A    L ONE
        STO  L LEN           ; LEN += 1
        MDX  1, 1            ; XR1 += 1
        BSC  L LOOP, 0       ; jump back

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
