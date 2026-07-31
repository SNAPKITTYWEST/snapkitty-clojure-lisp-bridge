/// Cranelift JIT compiler for SoulVM bytecode.
///
/// Translates a `SoulFunc` (stack-based bytecode) into native code via Cranelift's
/// SSA IR.  The value stack at each instruction is tracked at compile time;
/// each Instruction consumes/produces typed `cranelift::Value`s.

use cranelift_codegen::{
    ir::{
        condcodes::IntCC,
        types, AbiParam, InstBuilder, MemFlags, Signature, UserFuncName,
    },
    settings::{self, Configurable},
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

use crate::bytecode::{Instruction, SoulFunc, ValType};

/// A compiled, executable soul function.
pub struct CompiledFunc {
    /// Raw function pointer — caller must cast to the correct signature.
    pub code:  *const u8,
    /// Number of parameters.
    pub arity: usize,
}

unsafe impl Send for CompiledFunc {}
unsafe impl Sync for CompiledFunc {}

/// The JIT engine.  Owns the `JITModule` and all compiled code.
pub struct JitEngine {
    module: JITModule,
}

impl JitEngine {
    pub fn new() -> Result<Self, String> {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        flag_builder.set("opt_level", "speed").unwrap();
        let flags = settings::Flags::new(flag_builder);
        let isa = cranelift_native::builder()
            .map_err(|e| format!("native ISA: {e}"))?
            .finish(flags)
            .map_err(|e| format!("ISA finish: {e}"))?;

        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);
        Ok(Self { module })
    }

    /// Compile `soul_func` and return a callable `CompiledFunc`.
    pub fn compile(&mut self, soul_func: &SoulFunc) -> Result<CompiledFunc, String> {
        let sig = self.build_signature(soul_func);
        let func_id = self
            .module
            .declare_function(&soul_func.name, Linkage::Export, &sig)
            .map_err(|e| format!("declare_function: {e}"))?;

        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;
        ctx.func.name = UserFuncName::user(0, func_id.as_u32());

        let mut func_ctx = FunctionBuilderContext::new();
        {
            let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
            Translator::new(&mut bcx, soul_func).translate()?;
            bcx.finalize();
        }

        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| format!("define_function: {e}"))?;
        self.module.clear_context(&mut ctx);
        self.module
            .finalize_definitions()
            .map_err(|e| format!("finalize: {e}"))?;

        let code = self.module.get_finalized_function(func_id);
        Ok(CompiledFunc { code: code as *const u8, arity: soul_func.params.len() })
    }

    fn build_signature(&self, f: &SoulFunc) -> Signature {
        let mut sig = self.module.make_signature();
        for ty in &f.params {
            sig.params.push(AbiParam::new(to_cl_type(*ty)));
        }
        for ty in &f.returns {
            sig.returns.push(AbiParam::new(to_cl_type(*ty)));
        }
        sig
    }
}

fn to_cl_type(ty: ValType) -> cranelift_codegen::ir::Type {
    match ty {
        ValType::I64 => types::I64,
        ValType::F64 => types::F64,
    }
}

/// Per-function compilation state.
struct Translator<'a, 'b> {
    bcx:        &'a mut FunctionBuilder<'b>,
    func:       &'a SoulFunc,
    stack:      Vec<cranelift_codegen::ir::Value>,
    vars:       Vec<Variable>,
    terminated: bool, // true after emitting a block-terminating instruction
}

impl<'a, 'b> Translator<'a, 'b> {
    fn new(bcx: &'a mut FunctionBuilder<'b>, func: &'a SoulFunc) -> Self {
        Self { bcx, func, stack: vec![], vars: vec![], terminated: false }
    }

    fn translate(&mut self) -> Result<(), String> {
        // Declare one Cranelift Variable per local (params + extra locals).
        // In Cranelift 0.132, declare_var auto-assigns and returns the Variable.
        let all_locals: Vec<ValType> = self.func.params.iter()
            .chain(self.func.locals.iter())
            .copied()
            .collect();

        for ty in all_locals.iter() {
            let var = self.bcx.declare_var(to_cl_type(*ty));
            self.vars.push(var);
        }

        // Entry block.
        let entry = self.bcx.create_block();
        self.bcx.append_block_params_for_function_params(entry);
        self.bcx.switch_to_block(entry);
        self.bcx.seal_block(entry);

        // Bind function parameters to their Variables.
        let params: Vec<_> = self.bcx.block_params(entry).to_vec();
        for (i, val) in params.iter().enumerate() {
            self.bcx.def_var(self.vars[i], *val);
        }

        // Pre-scan for branch targets so we can create blocks before emitting jumps.
        // We collect targets first to avoid borrow conflicts with self.bcx.
        let branch_targets: Vec<Option<usize>> = self.func.body.iter().map(|instr| {
            match instr {
                Instruction::Br(t) | Instruction::BrIf(t) | Instruction::BrIfZ(t) => Some(*t),
                _ => None,
            }
        }).collect();

        let mut instr_blocks: Vec<Option<cranelift_codegen::ir::Block>> =
            vec![None; self.func.body.len()];

        for maybe_target in &branch_targets {
            if let Some(target) = maybe_target {
                if instr_blocks[*target].is_none() {
                    instr_blocks[*target] = Some(self.bcx.create_block());
                }
            }
        }

        // Emit all instructions.
        for instr in self.func.body.iter() {
            self.emit(instr)?;
        }

        // If no terminating instruction was emitted, add a default return.
        if !self.terminated {
            if self.func.returns.is_empty() {
                self.bcx.ins().return_(&[]);
            } else if let Some(val) = self.stack.last().copied() {
                self.bcx.ins().return_(&[val]);
            } else {
                let zero = self.bcx.ins().iconst(types::I64, 0);
                self.bcx.ins().return_(&[zero]);
            }
        }

        Ok(())
    }

    fn pop(&mut self) -> Result<cranelift_codegen::ir::Value, String> {
        self.stack.pop().ok_or_else(|| "stack underflow".to_string())
    }

    fn get_var(&self, idx: usize) -> Result<Variable, String> {
        self.vars.get(idx).copied().ok_or_else(|| format!("local {idx} out of range"))
    }

    fn emit(&mut self, instr: &Instruction) -> Result<(), String> {
        match instr {
            // ── Constants ────────────────────────────────────────────────────
            Instruction::I64Const(v) => {
                let val = self.bcx.ins().iconst(types::I64, *v);
                self.stack.push(val);
            }
            Instruction::F64Const(v) => {
                let val = self.bcx.ins().f64const(*v);
                self.stack.push(val);
            }

            // ── Integer arithmetic ────────────────────────────────────────────
            Instruction::I64Add => {
                let b = self.pop()?; let a = self.pop()?;
                let v = self.bcx.ins().iadd(a, b); self.stack.push(v);
            }
            Instruction::I64Sub => {
                let b = self.pop()?; let a = self.pop()?;
                let v = self.bcx.ins().isub(a, b); self.stack.push(v);
            }
            Instruction::I64Mul => {
                let b = self.pop()?; let a = self.pop()?;
                let v = self.bcx.ins().imul(a, b); self.stack.push(v);
            }
            Instruction::I64DivS => {
                let b = self.pop()?; let a = self.pop()?;
                let v = self.bcx.ins().sdiv(a, b); self.stack.push(v);
            }
            Instruction::I64And => {
                let b = self.pop()?; let a = self.pop()?;
                let v = self.bcx.ins().band(a, b); self.stack.push(v);
            }
            Instruction::I64Or => {
                let b = self.pop()?; let a = self.pop()?;
                let v = self.bcx.ins().bor(a, b); self.stack.push(v);
            }
            Instruction::I64Xor => {
                let b = self.pop()?; let a = self.pop()?;
                let v = self.bcx.ins().bxor(a, b); self.stack.push(v);
            }
            Instruction::I64Shl => {
                let b = self.pop()?; let a = self.pop()?;
                let v = self.bcx.ins().ishl(a, b); self.stack.push(v);
            }
            Instruction::I64ShrS => {
                let b = self.pop()?; let a = self.pop()?;
                let v = self.bcx.ins().sshr(a, b); self.stack.push(v);
            }

            // ── Integer comparisons → i64 (1 or 0) ───────────────────────────
            Instruction::I64Eq => {
                let b = self.pop()?; let a = self.pop()?;
                let bv = self.bcx.ins().icmp(IntCC::Equal, a, b);
                let v  = self.bcx.ins().uextend(types::I64, bv);
                self.stack.push(v);
            }
            Instruction::I64Ne => {
                let b = self.pop()?; let a = self.pop()?;
                let bv = self.bcx.ins().icmp(IntCC::NotEqual, a, b);
                let v  = self.bcx.ins().uextend(types::I64, bv);
                self.stack.push(v);
            }
            Instruction::I64LtS => {
                let b = self.pop()?; let a = self.pop()?;
                let bv = self.bcx.ins().icmp(IntCC::SignedLessThan, a, b);
                let v  = self.bcx.ins().uextend(types::I64, bv);
                self.stack.push(v);
            }
            Instruction::I64GtS => {
                let b = self.pop()?; let a = self.pop()?;
                let bv = self.bcx.ins().icmp(IntCC::SignedGreaterThan, a, b);
                let v  = self.bcx.ins().uextend(types::I64, bv);
                self.stack.push(v);
            }
            Instruction::I64LeS => {
                let b = self.pop()?; let a = self.pop()?;
                let bv = self.bcx.ins().icmp(IntCC::SignedLessThanOrEqual, a, b);
                let v  = self.bcx.ins().uextend(types::I64, bv);
                self.stack.push(v);
            }
            Instruction::I64GeS => {
                let b = self.pop()?; let a = self.pop()?;
                let bv = self.bcx.ins().icmp(IntCC::SignedGreaterThanOrEqual, a, b);
                let v  = self.bcx.ins().uextend(types::I64, bv);
                self.stack.push(v);
            }

            // ── Float arithmetic ─────────────────────────────────────────────
            Instruction::F64Add => {
                let b = self.pop()?; let a = self.pop()?;
                let v = self.bcx.ins().fadd(a, b); self.stack.push(v);
            }
            Instruction::F64Sub => {
                let b = self.pop()?; let a = self.pop()?;
                let v = self.bcx.ins().fsub(a, b); self.stack.push(v);
            }
            Instruction::F64Mul => {
                let b = self.pop()?; let a = self.pop()?;
                let v = self.bcx.ins().fmul(a, b); self.stack.push(v);
            }
            Instruction::F64Div => {
                let b = self.pop()?; let a = self.pop()?;
                let v = self.bcx.ins().fdiv(a, b); self.stack.push(v);
            }

            // ── Locals ────────────────────────────────────────────────────────
            Instruction::LocalGet(idx) => {
                let var = self.get_var(*idx as usize)?;
                let val = self.bcx.use_var(var);
                self.stack.push(val);
            }
            Instruction::LocalSet(idx) => {
                let val = self.pop()?;
                let var = self.get_var(*idx as usize)?;
                self.bcx.def_var(var, val);
            }

            // ── GC heap ───────────────────────────────────────────────────────
            Instruction::Load { offset } => {
                let ptr = self.pop()?;
                let val = self.bcx.ins().load(types::I64, MemFlags::new(), ptr, *offset as i32);
                self.stack.push(val);
            }
            Instruction::Store { offset } => {
                let val = self.pop()?;
                let ptr = self.pop()?;
                self.bcx.ins().store(MemFlags::new(), val, ptr, *offset as i32);
            }

            // ── Control flow ──────────────────────────────────────────────────
            Instruction::Return => {
                if self.func.returns.is_empty() {
                    self.bcx.ins().return_(&[]);
                } else {
                    let val = self.pop()?;
                    self.bcx.ins().return_(&[val]);
                }
                self.terminated = true;
            }
            Instruction::Nop => {}

            // ── Capability ops (trap stub — full impl in Phase 2) ─────────────
            Instruction::CapCheck { .. } | Instruction::CapInvoke { .. } => {
                self.bcx.ins().trap(cranelift_codegen::ir::TrapCode::user(1).unwrap());
                self.terminated = true;
            }

            // Br/Call reserved for Phase 2 CFG builder
            Instruction::Br(_) | Instruction::BrIf(_) | Instruction::BrIfZ(_) |
            Instruction::Call(_) | Instruction::Alloc { .. } => {
                return Err(format!("{instr:?} not yet supported in single-block JIT"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::Instruction::*;

    fn engine() -> JitEngine {
        JitEngine::new().expect("JitEngine::new")
    }

    #[test]
    fn jit_identity() {
        let mut f = SoulFunc::new("identity", vec![ValType::I64], vec![ValType::I64]);
        f.push(LocalGet(0)).push(Return);
        let compiled = engine().compile(&f).expect("compile");
        let func: extern "C" fn(i64) -> i64 = unsafe { std::mem::transmute(compiled.code) };
        assert_eq!(func(42), 42);
        assert_eq!(func(-7), -7);
    }

    #[test]
    fn jit_add_two_params() {
        let mut f = SoulFunc::new(
            "add_two",
            vec![ValType::I64, ValType::I64],
            vec![ValType::I64],
        );
        f.push(LocalGet(0)).push(LocalGet(1)).push(I64Add).push(Return);
        let compiled = engine().compile(&f).expect("compile");
        let func: extern "C" fn(i64, i64) -> i64 = unsafe { std::mem::transmute(compiled.code) };
        assert_eq!(func(3, 4), 7);
        assert_eq!(func(100, -50), 50);
    }

    #[test]
    fn jit_const_return() {
        let mut f = SoulFunc::new("const99", vec![], vec![ValType::I64]);
        f.push(I64Const(99)).push(Return);
        let compiled = engine().compile(&f).expect("compile");
        let func: extern "C" fn() -> i64 = unsafe { std::mem::transmute(compiled.code) };
        assert_eq!(func(), 99);
    }

    #[test]
    fn jit_mul_then_sub() {
        // (a * b) - c
        let mut f = SoulFunc::new(
            "mul_sub",
            vec![ValType::I64, ValType::I64, ValType::I64],
            vec![ValType::I64],
        );
        f.push(LocalGet(0))
         .push(LocalGet(1))
         .push(I64Mul)
         .push(LocalGet(2))
         .push(I64Sub)
         .push(Return);
        let compiled = engine().compile(&f).expect("compile");
        let func: extern "C" fn(i64, i64, i64) -> i64 = unsafe { std::mem::transmute(compiled.code) };
        assert_eq!(func(3, 4, 2), 10); // 3*4 - 2 = 10
        assert_eq!(func(5, 5, 0), 25);
    }

    #[test]
    fn jit_cmp_eq() {
        let mut f = SoulFunc::new(
            "eq_check",
            vec![ValType::I64, ValType::I64],
            vec![ValType::I64],
        );
        f.push(LocalGet(0)).push(LocalGet(1)).push(I64Eq).push(Return);
        let compiled = engine().compile(&f).expect("compile");
        let func: extern "C" fn(i64, i64) -> i64 = unsafe { std::mem::transmute(compiled.code) };
        assert_eq!(func(7, 7), 1);
        assert_eq!(func(7, 8), 0);
    }

    #[test]
    fn jit_f64_add() {
        let mut f = SoulFunc::new(
            "f64_add",
            vec![ValType::F64, ValType::F64],
            vec![ValType::F64],
        );
        f.push(LocalGet(0)).push(LocalGet(1)).push(F64Add).push(Return);
        let compiled = engine().compile(&f).expect("compile");
        let func: extern "C" fn(f64, f64) -> f64 = unsafe { std::mem::transmute(compiled.code) };
        assert!((func(1.5, 2.5) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn jit_bitwise_xor() {
        let mut f = SoulFunc::new(
            "xor",
            vec![ValType::I64, ValType::I64],
            vec![ValType::I64],
        );
        f.push(LocalGet(0)).push(LocalGet(1)).push(I64Xor).push(Return);
        let compiled = engine().compile(&f).expect("compile");
        let func: extern "C" fn(i64, i64) -> i64 = unsafe { std::mem::transmute(compiled.code) };
        assert_eq!(func(0b1100, 0b1010), 0b0110);
    }
}
