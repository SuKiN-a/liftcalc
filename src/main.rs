mod parser;
mod utils;
use chumsky::Parser;
use cranelift::{
    codegen::{
        ir::Function,
        settings::{self, Configurable},
    },
    prelude::*,
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module};
use parser::{parser, Expr};
use utils::UnsupportedMachineError;

fn main() -> anyhow::Result<()> {
    let isa = {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false")?;
        flag_builder.set("is_pic", "false")?;
        let isa_builder = cranelift_native::builder().map_err(UnsupportedMachineError)?;
        isa_builder.finish(settings::Flags::new(flag_builder))?
    };

    let mut module = JITModule::new(JITBuilder::with_isa(isa, default_libcall_names()));
    let mut func_ctx = FunctionBuilderContext::new();

    let signature = {
        let mut signature = module.make_signature();
        signature.returns.push(AbiParam::new(types::F64));
        signature
    };

    let function = module.declare_function("testfunc", Linkage::Local, &signature)?;

    let mut ctx = module.make_context();
    ctx.func.signature = signature;
    ctx.func.name = ExternalName::user(0, function.as_u32());
    build_function(&mut ctx.func, &mut func_ctx);
    module.define_function(function, &mut ctx)?;
    module.clear_context(&mut ctx);
    module.finalize_definitions();

    let result =
        unsafe { std::mem::transmute::<_, fn() -> f64>(module.get_finalized_function(function))() };
    println!("{result}");
    Ok(())
}

fn build_function(func: &mut Function, func_ctx: &mut FunctionBuilderContext) {
    let mut bcx = FunctionBuilder::new(func, func_ctx);
    let block = bcx.create_block();
    bcx.switch_to_block(block);
    let calc = std::env::args().nth(1).unwrap();
    let expr = parser().parse(calc).unwrap();
    let result = build_from_ast(&expr, &mut bcx);
    bcx.ins().return_(&[result]);
    bcx.seal_all_blocks();
    bcx.finalize();
}

fn build_from_ast(expr: &Expr, bcx: &mut FunctionBuilder) -> Value {
    match expr {
        Expr::Num(num) => bcx.ins().f64const(*num),
        Expr::Neg(expr) => {
            let val = build_from_ast(expr, bcx);
            bcx.ins().fneg(val)
        }
        Expr::Add(expr1, expr2) => {
            let val1 = build_from_ast(expr1, bcx);
            let val2 = build_from_ast(expr2, bcx);
            bcx.ins().fadd(val1, val2)
        }
        Expr::Sub(expr1, expr2) => {
            let val1 = build_from_ast(expr1, bcx);
            let val2 = build_from_ast(expr2, bcx);
            bcx.ins().fsub(val1, val2)
        }
        Expr::Mul(expr1, expr2) => {
            let val1 = build_from_ast(expr1, bcx);
            let val2 = build_from_ast(expr2, bcx);
            bcx.ins().fmul(val1, val2)
        }
        Expr::Div(expr1, expr2) => {
            let val1 = build_from_ast(expr1, bcx);
            let val2 = build_from_ast(expr2, bcx);
            bcx.ins().fdiv(val1, val2)
        }
    }
}
