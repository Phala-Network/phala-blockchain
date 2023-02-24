// File: types.h
//
// This file defines the pink runtime v1 interface types.
// We use a C header file to make sure that the types are ffi-safe.

#include <inttypes.h>
#include <stddef.h>

typedef void (*output_fn_t)(void *ctx, const uint8_t *data, size_t len);
typedef void (*cross_call_fn_t)(uint32_t call_id, const uint8_t *data, size_t len, void *ctx, output_fn_t output);

typedef struct
{
    // Whether the runtime is running in a dylib or compiled into the running binary.
    // If it is a dylib, the runtime will init logger inside.
    int is_dylib;
    // If true, the logger inside will be sanitized.
    int enclaved;
    cross_call_fn_t ocall;
} config_t;

typedef struct
{
    cross_call_fn_t ecall;
} ecalls_t;

typedef int init_t(const config_t *config, ecalls_t *ecalls);