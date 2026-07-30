; SKC-LISP-WORLD: Mutation Validation Gate (8-point validator)
; x64 NASM assembly
; Fast-path validation for mutation operations
;
; CALLING CONVENTION (System V AMD64 ABI):
;   rdi = pointer to mutation_event struct
;   rsi = pointer to object_store
;   rdx = pointer to validation_result (output)
;
; mutation_event struct layout (64 bytes):
;   [0]   mutation_id (u64)
;   [8]   generation_before (u64)
;   [16]  generation_after (u64)
;   [24]  actor (u64)
;   [32]  target (u64)
;   [40]  operation (u32)
;   [44]  reserved (u32)
;   [48]  old_digest (u64 ptr)
;   [56]  new_digest (u64 ptr)
;
; validation_result struct:
;   [0]   passes_gate (u8, 1=pass, 0=fail)
;   [1]   error_code (u8, 0-7 for check 1-8, 255=pass)

global mutation_validate_gate
section .text

mutation_validate_gate:
    ; prologue
    push rbx
    push r12
    push r13
    push r14
    sub rsp, 16

    ; Save parameters
    mov r12, rdi            ; r12 = mutation_event*
    mov r13, rsi            ; r13 = object_store*
    mov r14, rdx            ; r14 = validation_result*

    xor eax, eax            ; eax = check counter (0-8)
    xor ecx, ecx            ; ecx = error code (0 if all pass)

    ; CHECK 1: target exists or allocation requested
    mov r8, [r12 + 32]      ; r8 = target object_id
    cmp r8, 0
    je .check2              ; if target == 0, skip existence check (allocation case)
    mov rax, [r13]          ; rax = object_store->count
    cmp r8, rax
    jge .fail1              ; if target >= count, object doesn't exist → fail
    jmp .check2

.fail1:
    mov ecx, 1              ; error_code = 1 (check 1 failed)
    jmp .exit

    ; CHECK 2: old_digest matches (simplified: check non-null)
.check2:
    mov r8, [r12 + 48]      ; r8 = old_digest pointer
    cmp r8, 0
    je .fail2               ; if null, fail
    jmp .check3

.fail2:
    mov ecx, 2
    jmp .exit

    ; CHECK 3: new_digest matches (simplified: check non-null)
.check3:
    mov r8, [r12 + 56]      ; r8 = new_digest pointer
    cmp r8, 0
    je .fail3               ; if null, fail
    jmp .check4

.fail3:
    mov ecx, 3
    jmp .exit

    ; CHECK 4: replacement well-formed (always true, placeholder)
.check4:
    jmp .check5

    ; CHECK 5: all references valid (always true, placeholder)
.check5:
    jmp .check6

    ; CHECK 6: code is valid (always true, placeholder)
.check6:
    jmp .check7

    ; CHECK 7: invariants preserved (always true, placeholder)
.check7:
    jmp .check8

    ; CHECK 8: generation advances
.check8:
    mov r8, [r12 + 8]       ; r8 = generation_before
    mov r9, [r12 + 16]      ; r9 = generation_after
    cmp r9, r8
    jle .fail8              ; if generation_after <= generation_before, fail
    jmp .pass

.fail8:
    mov ecx, 8
    jmp .exit

.pass:
    mov ecx, 255            ; error_code = 255 (all checks pass)

.exit:
    ; Write result
    cmp ecx, 255
    je .success
    mov byte [r14 + 0], 0   ; passes_gate = 0
    mov byte [r14 + 1], cl  ; error_code = cl
    xor eax, eax
    jmp .return

.success:
    mov byte [r14 + 0], 1   ; passes_gate = 1
    mov byte [r14 + 1], 255 ; error_code = 255
    mov eax, 1              ; return 1

.return:
    ; epilogue
    add rsp, 16
    pop r14
    pop r13
    pop r12
    pop rbx
    ret


; ============================================================================
; CHECK: generation_monotonicity (helper for commit_generation)
; ============================================================================
global check_generation_monotonic
section .text

check_generation_monotonic:
    ; rdi = generation_before (u64)
    ; rsi = generation_after (u64)
    ; return: 1 if after > before, 0 otherwise

    cmp rsi, rdi
    jle .not_monotonic
    mov eax, 1
    ret

.not_monotonic:
    xor eax, eax
    ret
