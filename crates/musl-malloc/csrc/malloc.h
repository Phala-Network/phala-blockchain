#ifndef _MUSL_MALLOC_H
#define _MUSL_MALLOC_H

#include <stddef.h>

void *musl_realloc(void *p, size_t n);
void *musl_memalign(size_t align, size_t len);
void musl_free(void *p);
void *musl_malloc(size_t n);
void *musl_calloc(size_t m, size_t n);
#endif