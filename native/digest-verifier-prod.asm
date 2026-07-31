; SKC-LISP-WORLD: Digest Verification (PRODUCTION)
; x64 NASM assembly + libblake3 + libsodium linkage
; Real cryptographic digest verification for production
;
; LINKING:
;   gcc -c digest-verifier-prod.asm -o digest-verifier-prod.o
;   gcc digest-verifier-prod.o -o digest-verifier -lblake3 -lsodium
;
; CALLING CONVENTION (System V AMD64 ABI):
;   rdi = pointer to payload (bytes)
;   rsi = payload_length (u64)
;   rdx = pointer to expected_digest (32 bytes for Blake3, 64 for Ed25519)
;   rcx = pointer to public_key (32 bytes, Ed25519 only)
;   r8  = pointer to verification_result (output)

extern blake3_hasher_init
extern blake3_hasher_update
extern blake3_hasher_finalize
extern crypto_sign_open

global blake3_verify
global ed25519_verify

section .data
    blake3_hash_size equ 32     ; BLAKE3 produces 32-byte hash
    ed25519_sig_size equ 64     ; Ed25519 signature is 64 bytes
    ed25519_key_size equ 32     ; Ed25519 public key is 32 bytes

section .text

; ============================================================================
; BLAKE3 VERIFICATION (production)
; ============================================================================
blake3_verify:
    ; rdi = payload*
    ; rsi = payload_length
    ; rdx = expected_digest* (32 bytes)
    ; r8  = verification_result*

    push rbp
    mov rbp, rsp
    sub rsp, 64                 ; stack space for blake3_hasher struct + output

    ; Validate inputs
    test rdi, rdi
    jz .blake3_invalid_input
    test rsi, rsi
    jz .blake3_invalid_input
    test rdx, rdx
    jz .blake3_invalid_input
    test r8, r8
    jz .blake3_invalid_input

    ; Save parameters
    mov r9, rdi                 ; r9 = payload*
    mov r10, rsi                ; r10 = payload_length
    mov r11, rdx                ; r11 = expected_digest*
    mov r12, r8                 ; r12 = verification_result*

    ; Initialize blake3_hasher on stack
    lea rax, [rbp - 64]         ; rax = hasher_buffer*
    mov rdi, rax
    call blake3_hasher_init

    ; Update hasher with payload
    lea rdi, [rbp - 64]         ; rdi = hasher*
    mov rsi, r9                 ; rsi = payload*
    mov rdx, r10                ; rdx = payload_length
    call blake3_hasher_update

    ; Finalize hasher to get digest
    lea rdi, [rbp - 64]         ; rdi = hasher*
    lea rsi, [rbp - 32]         ; rsi = output_buffer* (on stack)
    mov rdx, 32                 ; rdx = output_len
    xor rcx, rcx                ; rcx = seek_offset (0)
    call blake3_hasher_finalize

    ; Compare computed digest with expected_digest
    lea rax, [rbp - 32]         ; rax = computed_digest*
    mov rsi, r11                ; rsi = expected_digest*

    ; Memory compare: 32 bytes
    xor rcx, rcx
    mov r13, 0                  ; match flag

.blake3_compare_loop:
    cmp rcx, 32
    je .blake3_compare_done
    mov al, byte [rax + rcx]
    mov bl, byte [rsi + rcx]
    cmp al, bl
    jne .blake3_mismatch
    inc rcx
    jmp .blake3_compare_loop

.blake3_compare_done:
    ; All bytes matched
    mov byte [r12 + 0], 1       ; digest_valid = 1
    mov byte [r12 + 1], 0       ; error_code = 0 (match)
    mov eax, 1
    jmp .blake3_return

.blake3_mismatch:
    mov byte [r12 + 0], 0       ; digest_valid = 0
    mov byte [r12 + 1], 1       ; error_code = 1 (mismatch)
    xor eax, eax
    jmp .blake3_return

.blake3_invalid_input:
    mov byte [r12 + 0], 0       ; digest_valid = 0
    mov byte [r12 + 1], 2       ; error_code = 2 (invalid_input)
    xor eax, eax

.blake3_return:
    add rsp, 64
    pop rbp
    ret

; ============================================================================
; ED25519 SIGNATURE VERIFICATION (production)
; ============================================================================
ed25519_verify:
    ; rdi = message*
    ; rsi = message_length
    ; rdx = signature* (64 bytes)
    ; rcx = public_key* (32 bytes)
    ; r8  = verification_result*

    push rbp
    mov rbp, rsp
    sub rsp, 32

    ; Validate inputs
    test rdi, rdi
    jz .ed25519_invalid_input
    test rdx, rdx
    jz .ed25519_invalid_input
    test rcx, rcx
    jz .ed25519_invalid_input
    test r8, r8
    jz .ed25519_invalid_input

    ; Save parameters
    mov r9, rdi                 ; r9 = message*
    mov r10, rsi                ; r10 = message_length
    mov r11, rdx                ; r11 = signature*
    mov r12, rcx                ; r12 = public_key*
    mov r13, r8                 ; r13 = verification_result*

    ; Call crypto_sign_open(message*, mlen_p, signature, message, message_length, public_key)
    ; Returns 0 on success, -1 on verification failure
    lea rax, [rbp - 8]          ; rax = output_length* (on stack)
    mov rdi, r9                 ; rdi = message* (output buffer)
    mov rsi, rax                ; rsi = mlen_p*
    mov rdx, r11                ; rdx = signature*
    mov rcx, r9                 ; rcx = message* (input)
    mov r8, r10                 ; r8 = message_length
    mov r9, r12                 ; r9 = public_key*
    call crypto_sign_open

    ; Check return value
    cmp rax, 0
    jne .ed25519_sig_invalid

    ; Signature valid
    mov byte [r13 + 0], 1       ; signature_valid = 1
    mov byte [r13 + 1], 0       ; error_code = 0 (valid)
    mov eax, 1
    jmp .ed25519_return

.ed25519_sig_invalid:
    mov byte [r13 + 0], 0       ; signature_valid = 0
    mov byte [r13 + 1], 1       ; error_code = 1 (invalid)
    xor eax, eax
    jmp .ed25519_return

.ed25519_invalid_input:
    mov byte [r13 + 0], 0       ; signature_valid = 0
    mov byte [r13 + 1], 2       ; error_code = 2 (invalid_input)
    xor eax, eax

.ed25519_return:
    add rsp, 32
    pop rbp
    ret
