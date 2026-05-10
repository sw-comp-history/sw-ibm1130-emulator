; Demo: arithmetic
;
; Computes (5 + 3) * 4 and stores the 32-bit product. Demonstrates
; LD, A (add), M (multiply -> ACC,EXT pair), and STD (store double
; word). After execution the two-word RESULT slot holds 32 in the
; low word; the high word is 0 because the product fits in 16 bits.
;
; The 1130's M instruction multiplies ACC * mem and produces a
; 32-bit product in the (ACC, EXT) pair: ACC holds the high word,
; EXT holds the low. STD writes the pair to consecutive memory
; words: RESULT gets ACC, RESULT+1 gets EXT.
;
; Memory layout:
;   word 0..N    code
;   FIVE         literal 5
;   THREE        literal 3
;   FOUR         literal 4
;   RESULT       output high word
;   RESULT+1     output low  word (= 32 for this demo)

        LD   L FIVE
        A    L THREE       ; ACC = 5 + 3 = 8
        M    L FOUR        ; (ACC, EXT) = 8 * 4 = 32 (32-bit product)
        STD  L RESULT      ; RESULT = ACC (high), RESULT+1 = EXT (low)
        WAIT

FIVE:   DC   5
THREE:  DC   3
FOUR:   DC   4
RESULT: DC   0             ; high word of 32-bit product
        DC   0             ; low  word of 32-bit product (= 32)
