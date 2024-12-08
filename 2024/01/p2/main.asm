%define OS_READ   0
%define OS_WRITE  1
%define OS_OPEN   2
%define OS_CLOSE  3
%define OS_MMAP   9
%define OS_MUNMAP 11
%define OS_EXIT   60
%define OS_CLOCK_GETTIME 228

%define O_RDONLY      0b000
%define O_WRONLY      0b001
%define O_RDWR        0b010
; there's more opts for dir, create, excl, noctty, nofollow...

%define PROT_NONE     0b000
%define PROT_READ     0b001
%define PROT_WRITE    0b010
%define PROT_EXEC     0b100

%define MAP_PRIVATE   0b10
; also more opts ig

%define CLOCKID_CLOCK_MONOTONIC 1

%define FD_STDIN  0
%define FD_STDOUT 1
%define FD_STDERR 2

; ---

section .bss
    ptos_str: resb 16
    cinstant: resb 128 ; (two i64, secs and nanos)
    prof_start_ns: resq 1

section .rodata
    PTR_0X: db "0x"
    NEWLINE: db 10
    ZERO_DEV: db "/dev/zero", 0

    MSG_NANOS_END: db "μs", 0x1b, "[0m", 10
    MSG_NANOS_END_LEN: equ $ - MSG_NANOS_END
    MSG_SPACE_IN_SPACE: db " in "
    MSG_SPACE_IN_SPACE_LEN: equ $ - MSG_SPACE_IN_SPACE
    MSG_ANSI_TOTAL: db 0x1b, "[1;35m", "Total "
    MSG_ANSI_TOTAL_LEN: equ $ - MSG_ANSI_TOTAL

    MSG_FOPENING_1: db 0x1b, "[36m", "Opening '"
    MSG_FOPENING_1_LEN: equ $ - MSG_FOPENING_1
    MSG_FOPENING_2: db "'...", 0x1b, "[0m", 10
    MSG_FOPENING_2_LEN: equ $ - MSG_FOPENING_2

    ERRMSG_NOARG: db 0x1b, "[1;31m"
                  db "Proper arguments not provided:", 10
                  db "  Usage: ./main <input path> <alignemnt>"
                  db 0x1b, "[0m", 10
    ERRMSG_NOARG_LEN: equ $ - ERRMSG_NOARG
    ERRMSG_FOPEN: db 0x1b, "[1;31m", "Error opening", 0x1b, "[0m", 10
    ERRMSG_FOPEN_LEN: equ $ - ERRMSG_FOPEN
    ERRMSG_MALLOC: db 0x1b, "[1;31m", "Error allocating memory", 0x1b, "[0m", 10
    ERRMSG_MALLOC_LEN: equ $ - ERRMSG_MALLOC
    ERRMSG_UNFULLREAD: db 0x1b, "[1;31m", "Read an unaligned ammount of bytes", 0x1b, "[0m", 10
    ERRMSG_UNFULLREAD_LEN: equ $ - ERRMSG_UNFULLREAD
    ERRMSG_GETTIME: db 0x1b, "[1;31m", "clock_gettime syscall failed", 0x1b, "[0m", 10
    ERRMSG_GETTIME_LEN: equ $ - ERRMSG_UNFULLREAD

    DBGMSG_READLN: db " - read an 'aligned' line", 10
    DBGMSG_READLN_LEN: equ $ - DBGMSG_READLN
    DBGMSG_READLNFNSH: db "finished reading lines", 10
    DBGMSG_READLNFNSH_LEN: equ $ - DBGMSG_READLNFNSH

section .data
    demo_arr: dq 1, 4, 2, 3, 4, 2
    demo_arr_len: equ $ - demo_arr

section .text
    global _start

_start:
    mov rbp, rsp
    sub rsp, 40
    ; [rbp-8]:  input fd
    ; [rbp-16]: alignment
    ; [rbp-24]: read buf
    ; [rbp-32]: position of stack before the array alloc thing
    ;  basically pseudo stack frame

    mov rax, [rbp] ; argc
    cmp rax, 3
    je .correct_args
        mov rsi, ERRMSG_NOARG
        mov rdx, ERRMSG_NOARG_LEN
        jmp errmsg
    .correct_args:

    mov rdi, [rbp+16] ; argv[1]
    call open_file
    mov [rbp-8], rax

    mov rsi, [rbp+24] ; argv[2]
    mov rdi, 10
    call stoi ; num in rax
    mov [rbp-16], rax

    call malloc ; takes rax too
    ; and we get ptr in rax
    mov [rbp-24], rax

    mov [rbp-32], rsp
    .read_loop:
        mov rax, OS_READ
        mov rdi, [rbp-8]
        mov rsi, [rbp-24]
        mov rdx, [rbp-16]
        syscall
        test rax, rax
        jz .read_loop_end
        cmp rax, [rbp-16]
        je .read_full_aligned
            mov rsi, ERRMSG_UNFULLREAD
            mov rdx, ERRMSG_UNFULLREAD_LEN
            jmp errmsg
        .read_full_aligned:

        ; so we pass an arg from stack
        ;  + reserving an additional entry
        sub rsp, 8
        push qword [rbp-24] ; ptr
        call line_parser
        ; and we dont restore stack position
        ; we store here until we know length
        ; to malloc and get it out

        jmp .read_loop
        .read_loop_end:


    ; and at this point we can redefine
    ;  [rbp-8]:  input fd
    ;  [rbp-16]: alignment
    ;  [rbp-24]: read buf
    ; but close all that before
    mov rax, OS_CLOSE
    mov rdi, [rbp-8]
    syscall
    mov rdi, [rbp-24]
    mov rsi, [rbp-16]
    call free
    ; and now
    ;  [rbp-8]:  list1 ptr
    ;  [rbp-16]: list2 ptr
    ;  [rbp-24]: each list size
        ; ignore dis
        call monotonic_now
        mov [prof_start_ns], rax

    ; nooow, we have all that shii in stack
    ; so we qalc the size (sub looks reversed bsc stack goes down)
    mov rax, [rbp-32]
    sub rax, rsp
    shr rax, 1 ; and divide by 2 ("double list")
    mov [rbp-24], rax

    call malloc
    mov [rbp-8], rax

    mov rax, [rbp-24]
    call malloc
    mov [rbp-16], rax


    ; and "deinterlace" lists from stack to the mem vecs
    mov rax, [rbp-24]
    mov rbx, [rbp-8]
    mov rcx, [rbp-16]
    .deint_loop:
        sub rax, 8

        pop r10
        pop r11
        mov [rbx+rax], r10
        mov [rcx+rax], r11

        test rax, rax
        jnz .deint_loop

    ; and now we can also redefine
    ; [rbp-32]: accumulator (rsp should be back to original place after popping that)
    mov qword [rbp-32], 0

    ; mov rax, demo_arr
    ; mov rsi, demo_arr_len
    ; mov rdi, 2
    ; call count_n
    ; mov rax, rsi
    ; call ptos
    ; call print_ptos

    ; iter index (reverse-iter) (size)
    mov r11, [rbp-24]
    .cmp_loop:
        sub r11, 8
        ; get next smolest in list1
        mov rax, [rbp-8]
        mov rsi, [rbp-24]
        mov rdi, [rax+r11]
        call count_n
        mov r12, rdi ; save num in r12

        mov rax, [rbp-16]
        mov rsi, [rbp-24]
        ; rdi preserved
        call count_n
        ; count in rsi

        mov rax, r12 ; first num
        mul rsi ; rdx:rax = rax * r/m64 (we just ignore rdx :P)
        add [rbp-32], rax

        test r11, r11
        jnz .cmp_loop

    call monotonic_now
    mov rdx, 0
    sub rax, [prof_start_ns]
    mov rdi, 1000
    div rdi
    push rax ; our perf in ns

    mov rsi, MSG_ANSI_TOTAL
    mov rdx, MSG_ANSI_TOTAL_LEN
    call print

    mov rax, [rbp-32]
    mov rdi, 10
    call itos
    mov rax, r10
    mov rdi, r9
    call just_print

    mov rsi, MSG_SPACE_IN_SPACE
    mov rdx, MSG_SPACE_IN_SPACE_LEN
    call print

    pop rax
    mov rdi, 10
    call itos
    mov rax, r10
    mov rdi, r9
    call just_print

    mov rsi, MSG_NANOS_END
    mov rdx, MSG_NANOS_END_LEN
    call print

    ; exit
    mov rdi, 0
    _exit:
    mov rax, OS_EXIT
    syscall

; count n in list
;  rax: list ptr
;  rsi: list size (bytes)
;  rdi: num to count
; returns:
;  rsi: amount
; [everything preserved but rsi and rdx]
count_n:
    push rbp
    mov rbp, rsp
    sub rsp, 8
    ; [rbp-8]: count

    mov qword [rbp-8], 0
    .count:
        sub rsi, 8
        mov rdx, [rax+rsi]
        cmp rdi, rdx
        jnz .after_add
            inc qword [rbp-8]
        .after_add:

        test rsi, rsi
        jnz .count

    mov rsi, [rbp-8]

    mov rsp, rbp
    pop rbp
    ret

monotonic_now:
    mov rax, OS_CLOCK_GETTIME
    mov rsi, cinstant
    mov rdi, CLOCKID_CLOCK_MONOTONIC
    syscall

    cmp rax, 0
    jl .noret
        mov rax, [cinstant+0] ; secs
        mov rdx, 1_000_000_000
        mul rdx ; we drop rdx, as always
        add rax, [cinstant+8] ; nanos
        ret
    .noret:
    mov rsi, ERRMSG_GETTIME
    mov rdx, ERRMSG_GETTIME_LEN
    jmp errmsg


; takes [rsp+8/rbp+16]: ptr
; returns:
;  [rsp+8 /rbp+16]: 1st num
;  [rsp+16/rbp+24]: 2nd num
line_parser:
    push rbp
    mov rbp, rsp
    sub rsp, 8 ; no need for alignment
    ; [rbp-8]: start of 2nd num

    ; nullify first space to stoi it
    mov rax, [rbp+16]
    .null_loop:
        inc rax
        cmp byte [rax], " "
        jne .null_loop
        mov byte [rax], 0
    ; now same but find the start
    .start_2nd_loop:
        inc rax
        cmp byte [rax], " "
        je .start_2nd_loop
        mov [rbp-8], rax
    ; and stoi also needs nullbyte, not newline
    .newline_loop:
        inc rax
        cmp byte [rax], 10 ; 10 = '\n'
        jne .newline_loop
        mov byte [rax], 0

    mov rdi, 10
    mov rsi, [rbp+16]
    call stoi
    mov [rbp+16], rax
    mov rsi, [rbp-8]
    call stoi
    mov [rbp+24], rax

    mov rsp, rbp
    pop rbp
    ret

; takes:
;  rdi: char* path
; returns rax: fd
open_file:
    push rbp
    mov rbp, rsp
    sub rsp, 8 ; reserve 8B

    mov qword [rbp-8], rdi

    mov rsi, MSG_FOPENING_1
    mov rdx, MSG_FOPENING_1_LEN
    call print
    mov rax, qword [rbp-8]
    call just_print_null
    mov rsi, MSG_FOPENING_2
    mov rdx, MSG_FOPENING_2_LEN
    call print

    mov rax, OS_OPEN
    mov rsi, O_RDONLY
    mov rdi, [rbp-8]
    mov rdx, 0
    syscall
    ; fd in rax

    cmp rax, 0
    jg .valid_fd ; rax > 0 (signed)
        mov rsi, ERRMSG_FOPEN
        mov rdx, ERRMSG_FOPEN_LEN
        jmp errmsg
    .valid_fd:

    mov rsp, rbp
    pop rbp
    ret

; takes:
;  rsi: errmsg ptr
;  rdx: errmsg len
errmsg:
    mov rdi, FD_STDERR
    call print_fd
    mov rdi, 1
    jmp _exit
print:
    mov rdi, FD_STDOUT
; same & rdi: fd
print_fd:
    mov rax, OS_WRITE
    syscall
    ret

; ---- printing
just_print_null:
    mov rdi, rax,
    .count_til_null:
        inc rdi
        movzx rdx, byte [rdi]
        test rdx, rdx
        jnz .count_til_null
    sub rdi, rax
just_print:
    mov rsi, rax
    mov rdx, rdi
    mov rax, OS_WRITE
    mov rdi, FD_STDOUT
    syscall
    ret

; ---- ptos
; pointer to string
; takes:
;  rax: pointer
; returns:
;  rsi: memory block (freeable when needed)
;  r10: start ptr (within block)
;  r9: str len
; garbage:
;  rax: consumed integer
;  rsi: 16 base :)
; notes:
;  div => rdx:rax / r/m64 => rax(quot), rdx(remnd)
ptos:
    call nullify_ptros_ptr
    ; save registers
    mov rsi, ptos_str
    mov rdi, 16
    ; align rsi ptr (we write from end)
    mov r10, rsi
    add r10, 16 ; we do an extra number cuz alignment and less intructs
    .ptos_iter:
        dec r10
        mov rdx, 0
        div rdi
        cmp rdx, 10
        jl .ptos_iter_after_alpha_shift
        add rdx, 7
        .ptos_iter_after_alpha_shift:
        add rdx, 48
        mov byte [r10], dl
        test rax,rax
        jnz .ptos_iter
    .ptos_return:
        mov r9, rsi
        add r9, 64
        sub r9, r10
        ret
nullify_ptros_ptr:
    push rax
    mov rax, 16
    .npp_iter:
        dec rax
        mov [ptos_str+rax], byte '0'
        test rax, rax
        jnz .npp_iter
    pop rax
    ret
print_ptos:
    mov rax, OS_WRITE
    mov rdi, FD_STDOUT
    mov rsi, PTR_0X
    mov rdx, 2
    syscall
    mov rax, OS_WRITE
    mov rsi, ptos_str
    mov rdx, 16
    syscall
print_newline:
    mov rax, OS_WRITE
    mov rdi, FD_STDOUT
    mov rsi, NEWLINE
    mov rdx, 1
    syscall
    ret


; -- malloc
; takes:
;  rax: size
; returns:
;  rax: addr
;
; /dev/null's fd gets lost tho
;
malloc:
    push rbp
    mov rbp, rsp
    sub rsp, 8   ; reserve 8B, no alignment needed

    mov qword [rbp-8], rax

    ; open /dev/null
    mov rax, OS_OPEN
    mov rdi, ZERO_DEV
    mov rsi, O_RDWR
    mov rdx, 0
    syscall
    ; fd in rax

    ; mmap it
    mov r8, rax  ; fd into r8
    mov r9, 0    ; "off"?
    mov rax, OS_MMAP
    mov rdi, 0
    mov rsi, [rbp-8]
    mov rdx, PROT_READ | PROT_WRITE
    mov r10, MAP_PRIVATE
    syscall

    mov rsp, rbp
    pop rbp
    ret

; -- free
; takes:
;  rdi: ptr
;  rsi: size
free:
    mov rax, OS_MUNMAP
    syscall
    ret

; -- stoi / itos
; TODO: moar than 10 radix

; string to interger (ik ppl call this atoi, but nvm)
; takes:
;  string pointer (null terminated): rsi
;  radix: rdi
; returns:
;  rax: result
; garbage:
;  rdx: last processed digit
;  rsi: ptr to str final nullbyte+1
; notes:
;  mul => rdx:rax = rax * r/m64
stoi:
    mov rax, 0
    .stoi_parse_digit:
        cmp byte [rsi], 0
        je .stoi_return      ; compare
        mul rdi              ; shift radix

        mov rdx, [rsi]
        and rdx, 0b1111 ; grab digit
        add rax, rdx    ; add digit to the number
        inc rsi
        jmp .stoi_parse_digit
    .stoi_return:
        ret

; integer to string (idk how ppl name this, but I already got stoi)
; takes:
;  rax: integer
;  rdi: radix
; returns:
;  rsi: memory block (freeable when needed)
;  r10: start ptr (within block)
;  r9: str len
; garbage:
;  rax: consumed integer
; notes:
;  div => rdx:rax / r/m64 => rax(quot), rdx(remnd)
itos:
    ; save registers
    push rax
    push rdi
    ; malloc some mem
    mov rax, 64     ; page size (max representable value is 64 bits long,
                  ; a full 64 bit register with radix 2)
    call malloc
    test rax, rax
    jnz .successful_malloc
        ; allocation error likely?
        mov rsi, ERRMSG_MALLOC
        mov rdx, ERRMSG_MALLOC_LEN
        jmp errmsg
    .successful_malloc:
    mov rsi, rax    ; put malloc'd mem into rsi
    ; restore registers
    pop rdi
    pop rax

    ; align rsi ptr (we write from end)
    mov r10, rsi
    add r10, 64 ; we do an extra number cuz alignment and less intructs
    .itos_iter:
        dec r10
        mov rdx, 0          ; is used as upper half of divide, so set to 0
        div rdi             ; now rax holds the new shifted result by default and we get
                        ; the remainder in rdx to shift into number range (higher
                        ; than 10 radix will work, but range of letters will be broken)
        add rdx, 48         ; '0' == 48
        mov byte [r10], dl  ; lower(8) rdx
        test rax,rax        ; gotta run at least once to at least get a zero when rax = 0
        jnz .itos_iter

    .itos_return:
        mov r9, rsi
        add r9, 64
        sub r9, r10
        ret

print_r10_r9:
print_last_itos:
    mov rax, OS_WRITE
    mov rdi, FD_STDOUT
    mov rsi, r10
    mov rdx, r9
    syscall
    ret
