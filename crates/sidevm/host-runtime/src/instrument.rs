use anyhow::{anyhow, Result};
use parity_wasm::elements::{Instruction, Module};
use wasm_instrument::gas_metering::{inject, MemoryGrowCost, Rules};

struct InstructionWeights {
    i64const: u32,
    i64load: u32,
    i64store: u32,
    select: u32,
    r#if: u32,
    br: u32,
    br_if: u32,
    br_table: u32,
    br_table_per_entry: u32,
    call: u32,
    call_indirect: u32,
    call_indirect_per_param: u32,
    local_get: u32,
    local_set: u32,
    local_tee: u32,
    global_get: u32,
    global_set: u32,
    memory_current: u32,
    memory_grow: u32,
    i64clz: u32,
    i64ctz: u32,
    i64popcnt: u32,
    i64eqz: u32,
    i64extendsi32: u32,
    i64extendui32: u32,
    i32wrapi64: u32,
    i64eq: u32,
    i64ne: u32,
    i64lts: u32,
    i64ltu: u32,
    i64gts: u32,
    i64gtu: u32,
    i64les: u32,
    i64leu: u32,
    i64ges: u32,
    i64geu: u32,
    i64add: u32,
    i64sub: u32,
    i64mul: u32,
    i64divs: u32,
    i64divu: u32,
    i64rems: u32,
    i64remu: u32,
    i64and: u32,
    i64or: u32,
    i64xor: u32,
    i64shl: u32,
    i64shrs: u32,
    i64shru: u32,
    i64rotl: u32,
    i64rotr: u32,
    f64const: u32,
    f64load: u32,
    f64store: u32,
    f64convert: u32,
    f64cmp: u32,
    f64low: u32,
    f64calc: u32,
}

impl InstructionWeights {
    // Values are taken from the pallet-contract
    const fn default_weights() -> Self {
        Self {
            i64const: 2960,
            i64load: 7280,
            i64store: 8360,
            select: 5980,
            r#if: 9990,
            br: 3060,
            br_if: 5770,
            br_table: 7170,
            br_table_per_entry: 40,
            call: 68540,
            call_indirect: 85180,
            call_indirect_per_param: 1760,
            local_get: 3050,
            local_set: 3900,
            local_tee: 3030,
            global_get: 9050,
            global_set: 11140,
            memory_current: 3640,
            memory_grow: 3640,
            i64clz: 3140,
            i64ctz: 3040,
            i64popcnt: 2970,
            i64eqz: 3160,
            i64extendsi32: 2890,
            i64extendui32: 2830,
            i32wrapi64: 3140,
            i64eq: 4740,
            i64ne: 4720,
            i64lts: 4680,
            i64ltu: 4690,
            i64gts: 4720,
            i64gtu: 4840,
            i64les: 4730,
            i64leu: 4710,
            i64ges: 4660,
            i64geu: 4690,
            i64add: 4450,
            i64sub: 4520,
            i64mul: 4520,
            i64divs: 11070,
            i64divu: 11620,
            i64rems: 11090,
            i64remu: 11730,
            i64and: 4500,
            i64or: 4480,
            i64xor: 4570,
            i64shl: 4740,
            i64shrs: 4680,
            i64shru: 4700,
            i64rotl: 4690,
            i64rotr: 4700,
            f64const: 2960,
            f64load: 7280,
            f64store: 8360,
            f64convert: 4700,
            f64cmp: 4700,
            f64low: 4700,
            f64calc: 21620,
        }
    }
}

impl InstructionWeights {
    fn rules<'a>(&'a self, module: &Module) -> InstrumentRules<'a> {
        InstrumentRules {
            weights: self,
            params: module
                .type_section()
                .iter()
                .flat_map(|section| section.types())
                .map(|func| {
                    let parity_wasm::elements::Type::Function(func) = func;
                    func.params().len() as u32
                })
                .collect(),
        }
    }
}

struct InstrumentRules<'a> {
    weights: &'a InstructionWeights,
    params: Vec<u32>,
}

impl Rules for InstrumentRules<'_> {
    fn instruction_cost(&self, instruction: &Instruction) -> Option<u32> {
        use Instruction::*;
        let w = &self.weights;
        let weight = match *instruction {
            End | Unreachable | Return | Else => 0,
            I32Const(_) | I64Const(_) | Block(_) | Loop(_) | Nop | Drop => w.i64const,
            I32Load(_, _)
            | I32Load8S(_, _)
            | I32Load8U(_, _)
            | I32Load16S(_, _)
            | I32Load16U(_, _)
            | I64Load(_, _)
            | I64Load8S(_, _)
            | I64Load8U(_, _)
            | I64Load16S(_, _)
            | I64Load16U(_, _)
            | I64Load32S(_, _)
            | I64Load32U(_, _) => w.i64load,
            I32Store(_, _)
            | I32Store8(_, _)
            | I32Store16(_, _)
            | I64Store(_, _)
            | I64Store8(_, _)
            | I64Store16(_, _)
            | I64Store32(_, _) => w.i64store,
            Select => w.select,
            If(_) => w.r#if,
            Br(_) => w.br,
            BrIf(_) => w.br_if,
            Call(_) => w.call,
            GetLocal(_) => w.local_get,
            SetLocal(_) => w.local_set,
            TeeLocal(_) => w.local_tee,
            GetGlobal(_) => w.global_get,
            SetGlobal(_) => w.global_set,
            CurrentMemory(_) => w.memory_current,
            GrowMemory(_) => w.memory_grow,
            CallIndirect(idx, _) => {
                let nargs = *self.params.get(idx as usize).unwrap_or(&128);
                w.call_indirect + w.call_indirect_per_param * nargs
            }
            BrTable(ref data) => w
                .br_table
                .saturating_add(w.br_table_per_entry.saturating_mul(data.table.len() as u32)),
            I32Clz | I64Clz => w.i64clz,
            I32Ctz | I64Ctz => w.i64ctz,
            I32Popcnt | I64Popcnt => w.i64popcnt,
            I32Eqz | I64Eqz => w.i64eqz,
            I64ExtendSI32 => w.i64extendsi32,
            I64ExtendUI32 => w.i64extendui32,
            I32WrapI64 => w.i32wrapi64,
            I32Eq | I64Eq => w.i64eq,
            I32Ne | I64Ne => w.i64ne,
            I32LtS | I64LtS => w.i64lts,
            I32LtU | I64LtU => w.i64ltu,
            I32GtS | I64GtS => w.i64gts,
            I32GtU | I64GtU => w.i64gtu,
            I32LeS | I64LeS => w.i64les,
            I32LeU | I64LeU => w.i64leu,
            I32GeS | I64GeS => w.i64ges,
            I32GeU | I64GeU => w.i64geu,
            I32Add | I64Add => w.i64add,
            I32Sub | I64Sub => w.i64sub,
            I32Mul | I64Mul => w.i64mul,
            I32DivS | I64DivS => w.i64divs,
            I32DivU | I64DivU => w.i64divu,
            I32RemS | I64RemS => w.i64rems,
            I32RemU | I64RemU => w.i64remu,
            I32And | I64And => w.i64and,
            I32Or | I64Or => w.i64or,
            I32Xor | I64Xor => w.i64xor,
            I32Shl | I64Shl => w.i64shl,
            I32ShrS | I64ShrS => w.i64shrs,
            I32ShrU | I64ShrU => w.i64shru,
            I32Rotl | I64Rotl => w.i64rotl,
            I32Rotr | I64Rotr => w.i64rotr,
            F32Load(_, _) | F64Load(_, _) => w.f64load,
            F32Store(_, _) | F64Store(_, _) => w.f64store,
            F32Const(_) | F64Const(_) => w.f64const,
            F32Eq | F32Ne | F32Lt | F32Gt | F32Le | F32Ge | F64Eq | F64Ne | F64Lt | F64Gt
            | F64Le | F64Ge | F32Min | F32Max | F64Min | F64Max => w.f64cmp,
            F32Abs | F32Neg | F32Ceil | F32Floor | F32Trunc | F32Nearest | F32Copysign | F64Abs
            | F64Neg | F64Ceil | F64Floor | F64Trunc | F64Nearest | F64Copysign => w.f64low,
            F32Sqrt | F32Add | F32Sub | F32Mul | F32Div | F64Sqrt | F64Add | F64Sub | F64Mul
            | F64Div => w.f64calc,
            I32TruncSF32 | I32TruncUF32 | I32TruncSF64 | I32TruncUF64 | I64TruncSF32
            | I64TruncUF32 | I64TruncSF64 | I64TruncUF64 | F32ConvertSI32 | F32ConvertUI32
            | F32ConvertSI64 | F32ConvertUI64 | F32DemoteF64 | F64ConvertSI32 | F64ConvertUI32
            | F64ConvertSI64 | F64ConvertUI64 | F64PromoteF32 | I32ReinterpretF32
            | I64ReinterpretF64 | F32ReinterpretI32 | F64ReinterpretI64 => w.f64convert,

            // Returning None makes the gas instrumentation fail which we intend for
            // unsupported or unknown instructions.
            #[allow(unreachable_patterns)]
            _ => {
                log::error!("Unsupported instruction: {:?}", instruction);
                return None;
            }
        };
        Some(weight)
    }

    fn memory_grow_cost(&self) -> MemoryGrowCost {
        // TODO.kevin: Charge for memory usage
        // We don't charge the memory by instrument.
        // We charge the memory fee by `total_memory_usage * memory_per_page_per_block * instance_runing_time`
        // in the sidevm runtime.
        MemoryGrowCost::Free
    }
}

pub fn instrument(wasm: &[u8]) -> Result<Vec<u8>> {
    const WEIGHTS: InstructionWeights = InstructionWeights::default_weights();
    let module = Module::from_bytes(wasm)?;
    let rules = WEIGHTS.rules(&module);
    let module = inject(module, &rules, "sidevm").map_err(|_| anyhow!("Invalid module"))?;
    Ok(module.into_bytes()?)
}
