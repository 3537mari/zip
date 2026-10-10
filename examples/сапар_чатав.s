; module: zip
; target-cpu=native (AVX2/AVX-512 enabled)
; opt-level=-O3
; debug=true
.intel_syntax noprefix
.text
.file 1 "zip.zip"
.loc 1 1 1

.globl zip_main
zip_main:

.globl zip_glavnaya
zip_glavnaya:
  push rbp
  mov rbp, rsp
  sub rsp, 32
  ; return_type: Несовершенный
  mov rcx, qword ptr [rbp - 8]
  call zip______
  mov qword ptr [rbp - 24], rax
  ; impulse/move noundef
  mov rax, qword ptr [rbp - 24]
  mov qword ptr [rbp - 32], rax
  mov qword ptr [rbp - 24], 0
  mov rcx, qword ptr [rbp - 32]
  call zip______
  mov qword ptr [rbp - 32], rax
  ; llvm.invariant.start r3 (PERFECT)
  ; memory readonly
  ; impulse/move noundef
  mov rax, qword ptr [rbp - 32]
  mov qword ptr [rbp - 8], rax
  mov qword ptr [rbp - 32], 0
  mov rax, qword ptr [rbp - 8]
  leave
  ret
.loc 1 1 1

.globl zip______
zip______:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  ; return_type: Несовершенный
  ; await impulse r1
  lfence
  mov rax, qword ptr [rbp - 16]
  leave
  ret
.loc 1 1 1

.globl zip______
zip______:
  push rbp
  mov rbp, rsp
  sub rsp, 48
  ; attribute: PERFECT (immutable)
  ; return_type: Совершенный
  ; await impulse r4
  lfence
  ; llvm.invariant.start r4 (PERFECT)
  ; memory readonly
  mov rax, qword ptr [rbp - 40]
  leave
  ret

; zip-opt: passes=simplifycfg,mem2reg,instcombine level=O3
