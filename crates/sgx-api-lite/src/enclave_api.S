    .global sgx_report
    .type sgx_report, @function
sgx_report:
    .cfi_startproc
    push   rbx
    push   rcx

    mov    rbx,rdi
    mov    rcx,rsi
    mov    rax,0x0
    clc
    enclu
    setc   al

    pop    rcx
    pop    rbx
    ret
    .cfi_endproc

    .global sgx_getkey
    .type sgx_getkey, @function
sgx_getkey:
    .cfi_startproc
    push   rbx
    push   rcx

    mov    rbx,rdi
    mov    rcx,rsi
    mov    rax,0x1
    enclu
    jz   .Legetkey_done
    xor  rax, rax

.Legetkey_done:
    pop    rcx
    pop    rbx
    ret
    .cfi_endproc
