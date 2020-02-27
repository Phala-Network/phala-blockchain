# Notest

## Fix missing C functions

```c++

// Orig:
void GFp_bn_mul_mont(BN_ULONG *rp, const BN_ULONG *ap, const BN_ULONG *bp,
                     const BN_ULONG *np, const BN_ULONG *n0, size_t num);
// where
// - BN_ULONG is u32
// - size_t is u32

// Let's try:
void GFp_bn_mul_mont(uint32_t *rp, const uint32_t *ap, const uint32_t *bp,
                     const uint32_t *np, const uint32_t *n0, size_t num) {
  // nop
}
```




All missing functions:

```lisp
  (import "env" "ext_hashing_blake2_256_version_1" (func $ext_hashing_blake2_256_version_1 (type 9)))
  (import "env" "ext_hashing_twox_128_version_1" (func $ext_hashing_twox_128_version_1 (type 9)))
  (import "env" "ext_storage_set_version_1" (func $ext_storage_set_version_1 (type 10)))
  (import "env" "ext_storage_clear_version_1" (func $ext_storage_clear_version_1 (type 11)))
  (import "env" "ext_storage_root_version_1" (func $ext_storage_root_version_1 (type 12)))
  (import "env" "ext_crypto_ed25519_verify_version_1" (func $ext_crypto_ed25519_verify_version_1 (type 13)))
  (import "env" "ext_crypto_sr25519_verify_version_1" (func $ext_crypto_sr25519_verify_version_1 (type 13)))
  (import "env" "ext_storage_clear_prefix_version_1" (func $ext_storage_clear_prefix_version_1 (type 11)))
  (import "env" "ext_misc_print_utf8_version_1" (func $ext_misc_print_utf8_version_1 (type 11)))
  (import "env" "ext_misc_print_num_version_1" (func $ext_misc_print_num_version_1 (type 11)))
  (import "env" "ext_misc_print_hex_version_1" (func $ext_misc_print_hex_version_1 (type 11)))
  (import "env" "ext_crypto_sr25519_generate_version_1" (func $ext_crypto_sr25519_generate_version_1 (type 14)))
  (import "env" "ext_crypto_ed25519_generate_version_1" (func $ext_crypto_ed25519_generate_version_1 (type 14)))
  (import "env" "LIMBS_are_even" (func $LIMBS_are_even (type 1)))
  (import "env" "LIMBS_less_than_limb" (func $LIMBS_less_than_limb (type 0)))
  (import "env" "GFp_bn_neg_inv_mod_r_u64" (func $GFp_bn_neg_inv_mod_r_u64 (type 15)))
  (import "env" "LIMB_shr" (func $LIMB_shr (type 1)))
  (import "env" "LIMBS_shl_mod" (func $LIMBS_shl_mod (type 16)))
  (import "env" "LIMBS_less_than" (func $LIMBS_less_than (type 0)))
  (import "env" "LIMBS_are_zero" (func $LIMBS_are_zero (type 1)))
  (import "env" "ext_storage_get_version_1" (func $ext_storage_get_version_1 (type 15)))
  (import "env" "ext_storage_read_version_1" (func $ext_storage_read_version_1 (type 17)))
  (import "env" "ext_storage_changes_root_version_1" (func $ext_storage_changes_root_version_1 (type 8)))
  (import "env" "ext_storage_blake2_256_ordered_trie_root_version_1" (func $ext_storage_blake2_256_ordered_trie_root_version_1 (type 9)))
  (import "env" "ext_crypto_secp256k1_ecdsa_recover_compressed_version_1" (func $ext_crypto_secp256k1_ecdsa_recover_compressed_version_1 (type 18)))
  (import "env" "ext_allocator_malloc_version_1" (func $ext_allocator_malloc_version_1 (type 3)))
  (import "env" "ext_allocator_free_version_1" (func $ext_allocator_free_version_1 (type 7)))
```


## System Headers

```c
// from include/GFp/base.h
#include <stddef.h>  // supported
#include <stdint.h>  // no, maybe replaceable
// from internal.h
#include <stdalign.h>  // supported
```


## Troubles

### clang6 bitcode can't be used in llvm9 (used by rustc)

```bash
error: linking with `rust-lld` failed: exit code: 1
  |
  = note: "rust-lld" "-flavor" "wasm" "--no-threads" "-z" "stack-size=1048576" "--stack-first" "--allow-undefined" "--fatal-warnings" "--no-demangle" "--export-dynamic" "--no-entry" "-L" "/home/h4x/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/wasm32-unknown-unknown/lib" "/media/disk2/workspace/staging/phala-blockchain/node/target/release/wbuild/target/wasm32-unknown-unknown/release/deps/phala_blockchain_runtime.phala_blockchain_runtime.16kkxr4k-cgu.0.rcgu.o" "-o" "/media/disk2/workspace/staging/phala-blockchain/node/target/release/wbuild/target/wasm32-unknown-unknown/release/deps/phala_blockchain_runtime.wasm" "--export" "Core_initialize_block" "--export" "BlockBuilder_apply_extrinsic" "--export" "OffchainWorkerApi_offchain_worker" "--export" "TaggedTransactionQueue_validate_transaction" "--export" "BlockBuilder_inherent_extrinsics" "--export" "AuraApi_authorities" "--export" "GrandpaApi_grandpa_authorities" "--export" "AuraApi_slot_duration" "--export" "Metadata_metadata" "--export" "BlockBuilder_check_inherents" "--export" "Core_version" "--export" "BlockBuilder_random_seed" "--export" "SessionKeys_generate_session_keys" "--export" "Core_execute_block" "--export" "BlockBuilder_finalize_block" "--export=__heap_base" "--export=__data_end" "--gc-sections" "-O3" "-L" "/media/disk2/workspace/staging/phala-blockchain/node/target/release/wbuild/target/wasm32-unknown-unknown/release/deps" "-L" "/media/disk2/workspace/staging/phala-blockchain/node/target/release/wbuild/target/release/deps" "-L" "/media/disk2/workspace/staging/phala-blockchain/node/target/release/wbuild/target/wasm32-unknown-unknown/release/build/ring-135668556cb91874/out" "-L" "/home/h4x/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/wasm32-unknown-unknown/lib" "/tmp/rustc44Bien/libring-fa9ff96974412e8f.rlib" "/home/h4x/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/wasm32-unknown-unknown/lib/libcompiler_builtins-0d157676f9fe98f4.rlib" "--no-entry" "--export-table" "--export=__heap_base"
  = note: !dbg attachment points at wrong subprogram for function
          !193 = distinct !DISubprogram(name: "LIMBS_equal_limb", scope: !29, file: !29, line: 47, type: !194, scopeLine: 47, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !28, retainedNodes: !196)
          i32 (i32*, i32, i32)* @LIMBS_equal_limb
            br i1 %37, label %38, label %21, !dbg !240, !llvm.loop !134
          !88 = !DILocation(line: 30, column: 3, scope: !81)
          !81 = distinct !DILexicalBlock(scope: !72, file: !29, line: 30, column: 3)
          !72 = distinct !DISubprogram(name: "LIMBS_are_zero", scope: !29, file: !29, line: 28, type: !73, scopeLine: 28, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !28, retainedNodes: !76)
          !dbg attachment points at wrong subprogram for function
          !221 = distinct !DISubprogram(name: "LIMBS_less_than_limb", scope: !29, file: !29, line: 83, type: !222, scopeLine: 83, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !28, retainedNodes: !224)
          i32 (i32*, i32, i32)* @LIMBS_less_than_limb
            br i1 %32, label %33, label %16, !dbg !260, !llvm.loop !134
          !88 = !DILocation(line: 30, column: 3, scope: !81)
          !81 = distinct !DILexicalBlock(scope: !72, file: !29, line: 30, column: 3)
          !72 = distinct !DISubprogram(name: "LIMBS_are_zero", scope: !29, file: !29, line: 28, type: !73, scopeLine: 28, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition | DISPFlagOptimized, unit: !28, retainedNodes: !76)
          LLVM ERROR: Broken module found, compilation aborted!
```