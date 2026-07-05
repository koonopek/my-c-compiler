.global _main
_main:
	mov w0, #2
	mvn w0, w0
	str w0, [sp, #-16]!
	mov w0, #3
	ldr w1, [sp], #16
	add w0, w1, w0

	ret
