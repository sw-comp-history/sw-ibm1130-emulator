; Demo: hello-world on the 1054/console Selectric printer.
;
; Walks a null-terminated message, calling XIO once per character
; to type each byte on the console printer. This is the same shape
; as real hand-coded 1130 console-output programs, modulo two
; pedagogical simplifications:
;
;   1. Message bytes are stored as ASCII (one byte per word, in the
;      low byte). Real 1130 software talking to the console
;      Selectric used EBCDIC; ASCII is used here so the source is
;      readable without an encoding table. See
;      gen-isa/docs/character-encoding-plan.md for the future EBCDIC
;      retrofit.
;
;   2. The IOCC (I/O Control Command) is hand-built rather than
;      pulled from a system subroutine library. Real programs would
;      typically call a system subroutine like WRTY0 (LIBF) which
;      builds the IOCC and issues the XIO; we open-code the
;      sequence so the demo is self-contained.
;
; IOCC layout (two words at IOCC):
;   IOCC+0  : address of the data word holding the byte to type
;   IOCC+1  : high byte = (area << 3) | function
;             For area=1 (console) and function=0 (WRITE), the high
;             byte = 0x08 -> word value 0x0800.

        LDX  L 1, ZERO          ; XR1 = 0  (offset into MSG)

LOOP:   LD   L 1, MSG           ; ACC = MSG[XR1]
        BSC  0x01               ; skip next instr if ACC == 0 (sentinel hit)
        BSC  L EMIT, 0          ; otherwise jump to the per-char emit block
        BSC  L END_OF_PROGRAM, 0 ; ACC == 0 -> jump to halt

EMIT:   STO  L IOCC_DATA        ; place the char byte in the IOCC's data slot
        XIO  L IOCC             ; type the byte on the console printer
        MDX  1, 1               ; XR1 += 1
        BSC  L LOOP, 0          ; loop back

END_OF_PROGRAM:
        WAIT

; --- IOCC and data area ---

IOCC:       DC   IOCC_DATA      ; word 0: address of the data byte
            DC   0x0800         ; word 1: area=1 (CONSOLE), function=0 (WRITE)
IOCC_DATA:  DC   0              ; one-word buffer (low byte = char)

ZERO:       DC   0

; --- Message: "HELLO, WORLD!" plus null terminator ---

MSG:        DC   0x48           ; H
            DC   0x45           ; E
            DC   0x4C           ; L
            DC   0x4C           ; L
            DC   0x4F           ; O
            DC   0x2C           ; ,
            DC   0x20           ; (space)
            DC   0x57           ; W
            DC   0x4F           ; O
            DC   0x52           ; R
            DC   0x4C           ; L
            DC   0x44           ; D
            DC   0x21           ; !
            DC   0x0A           ; (newline)
            DC   0              ; sentinel
