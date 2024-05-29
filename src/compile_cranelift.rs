use std::mem;

use cranelift::{
    codegen::{
        ir::{
            types::{I32, I64, I8},
            Function, StackSlot, UserFuncName,
        },
        isa::{CallConv, OwnedTargetIsa, TargetFrontendConfig},
        verify_function,
    },
    prelude::*,
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module};

use crate::bf_ir::{BfIrScope, BfIrTok, MAX_CELL_COUNT};

pub fn dev_run() {
    let shared_builder = settings::builder();
    let shared_flags = settings::Flags::new(shared_builder);

    let isa = isa::lookup(target_lexicon::triple!("x86_64")).unwrap();
    let isa = isa.finish(shared_flags).unwrap();

    println!("{}", isa);

    // let program = "++--.,.,";
    // let program = "++--.,.,[.]  [-]  [+[+]+[-]>>]";
    let program = "";
    let program = crate::bf::BfParser::new(program.as_bytes())
        .parse()
        .unwrap();
    dbg!(&program);
    let program = BfIrScope::from_bf(program);
    dbg!(&program);
    println!("{}", program);

    println!("=====");

    let f = compile(program, isa.frontend_config());
    std::fs::write("./bf_programs/compiled.clif", f.display().to_string()).unwrap();
    //
    println!("{}", f.display());
    run(f, isa.to_owned());
}

struct BuildCtx<'a, 'b> {
    builder: &'a mut FunctionBuilder<'b>,

    targ_cfg: TargetFrontendConfig,
    /// `ptr` offset from start of data array (unsigned)
    data_ptr: &'a StackSlot,
    /// The stack slot storing the data array
    data: &'a StackSlot,
    data_size: u32,
}

impl BuildCtx<'_, '_> {
    /// Gets a pointer to `self.data + self.data_ptr + offset`
    pub fn addr_of_data(&mut self, offset: i32) -> Value {
        let base = self
            .builder
            .ins()
            .stack_addr(self.targ_cfg.pointer_type(), *self.data, offset);
        // let ptr_offset = self.builder.use_var(*self.data_ptr);
        let ptr_offset =
            self.builder
                .ins()
                .stack_load(self.targ_cfg.pointer_type(), *self.data_ptr, 0);
        let ptr = self.builder.ins().iadd(base, ptr_offset);
        ptr
    }
    /// Loads the byte at `self.data + self.data_ptr + offset`
    pub fn load_data(&mut self, offset: i32) -> Value {
        let ptr = self.addr_of_data(offset);
        self.builder.ins().load(I8, MemFlags::new(), ptr, 0)
    }
    pub fn store_data(&mut self, val: Value, offset: i32) {
        let ptr = self.addr_of_data(offset);
        self.builder.ins().store(MemFlags::new(), val, ptr, 0);
    }
}

/// Turns a `BfIrScope` into a series of blocks, starting with `curr_block`
fn build_scope<'a>(sc: BfIrScope, ctx: &mut BuildCtx, curr_block: Block, return_to: Block) {
    ctx.builder.switch_to_block(curr_block);

    for tok in sc.as_ref() {
        match tok {
            // BfIrTok::Set(_) => todo!(),
            // BfIrTok::Add(delta) => {
            //     let old = ctx.load_data(0);
            //     let new = ctx.builder.ins().iadd_imm(old, i64::from(delta.0));
            //     ctx.store_data(new, 0);
            // }
            // BfIrTok::PtrAdd(_) => todo!(),
            BfIrTok::Modify { adds, ptr_delta } => todo!(),
            BfIrTok::Read => todo!(),
            BfIrTok::Write => todo!(),
            BfIrTok::Loop(inner) => {
                let pre_block = ctx.builder.create_block();
                let inner_block = ctx.builder.create_block();
                let post_block = ctx.builder.create_block();

                {
                    ctx.builder.ins().jump(pre_block, &[]);
                }

                // `return_to` = `pre_block` to perform looping
                build_scope(inner.clone(), ctx, inner_block, pre_block);

                ctx.builder.switch_to_block(pre_block);
                {
                    // If data == 0, skip (else)
                    // if data != 0, loop (then)
                    let cond = ctx.load_data(0);
                    ctx.builder
                        .ins()
                        .brif(cond, inner_block, &[], post_block, &[]);
                }

                ctx.builder.switch_to_block(post_block);
            }
        }
    }

    // Should be switched to `curr_block`
    {
        ctx.builder.ins().jump(return_to, &[]);
    }
}

pub fn compile(sc: BfIrScope, targ_cfg: TargetFrontendConfig) -> Function {
    let sig = Signature::new(CallConv::SystemV); // Our entrypoint has no arguments or returns
    let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);
    let mut func_builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut func_builder_ctx);

    let main_block = builder.create_block();
    let inner_block = builder.create_block();
    let post_main_block = builder.create_block();

    // let data_ptr = Variable::new(0);
    let data_ptr = builder.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        targ_cfg.pointer_bytes() as u32,
    ));
    let data = builder.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        MAX_CELL_COUNT as u32,
    ));

    builder.switch_to_block(main_block);
    {
        let zero8 = builder.ins().iconst(I8, 0);
        let zero_ptr = builder.ins().iconst(targ_cfg.pointer_type(), 0);
        builder.ins().stack_store(zero_ptr, data_ptr, 0);

        let base = builder.ins().stack_addr(targ_cfg.pointer_type(), data, 0);
        let size = builder.ins().iconst(
            targ_cfg.pointer_type(),
            i64::try_from(MAX_CELL_COUNT).unwrap(),
        );
        println!("{size:?}");
        builder.call_memset(targ_cfg, base, zero8, size);

        builder.ins().jump(inner_block, &[]);
    }

    build_scope(
        sc,
        &mut BuildCtx {
            builder: &mut builder,

            targ_cfg,
            data_ptr: &data_ptr,
            data: &data,
            data_size: MAX_CELL_COUNT as u32,
        },
        inner_block,
        post_main_block,
    );

    builder.switch_to_block(post_main_block);
    {
        builder.ins().return_(&[]);
    }

    builder.seal_all_blocks();
    builder.finalize();

    println!("{}", func.display());
    let flags = settings::Flags::new(settings::builder());
    verify_function(&func, &flags).unwrap();

    func
}

pub fn run(f: Function, isa: OwnedTargetIsa) {
    let mut module = JITModule::new(JITBuilder::with_isa(isa, default_libcall_names()));
    let mut ctx = module.make_context();
    let mut _sig = module.make_signature();
    _sig.call_conv = CallConv::SystemV;

    let fid = module
        .declare_function("main", Linkage::Local, &f.signature)
        .unwrap();
    ctx.func = f;

    module.define_function(fid, &mut ctx).unwrap();
    module.clear_context(&mut ctx);

    module.finalize_definitions().unwrap();

    let f_ptr = module.get_finalized_function(fid);
    let f_ptr = unsafe { mem::transmute::<_, extern "C" fn() -> ()>(f_ptr) };
    let _res = f_ptr();
}

pub fn foo(isa: OwnedTargetIsa) {
    let func = {
        let sig = Signature::new(CallConv::SystemV); // Our entrypoint has no arguments or returns
        let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);
        let mut func_builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut func_builder_ctx);

        let main_block = builder.create_block();

        builder.switch_to_block(main_block);
        {
            builder.ins().return_(&[]);
        }

        builder.seal_all_blocks();
        builder.finalize();

        func
    };

    run(func, isa);
}
