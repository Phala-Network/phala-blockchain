    .global sgx_report
    .type sgx_report, @function
sgx_report:
    mov    r10,rbx
    mov    eax,0x0
    mov    rbx,rdi
    mov    rcx,rsi
    enclu
    xor    rax,rax
    mov    rbx,r10
    ret

    .global sgx_getkey
    .type sgx_getkey, @function
sgx_getkey:
    mov    r10,rbx
    mov    eax,0x1
    mov    rbx,rdi
    mov    rcx,rsi
    enclu
    mov    rbx,r10
    ret
