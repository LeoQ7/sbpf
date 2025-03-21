#![no_main]

use std::hint::black_box;

use libfuzzer_sys::fuzz_target;

use solana_sbpf::{
    ebpf,
    elf::Executable,
    memory_region::MemoryRegion,
    program::{BuiltinFunction, BuiltinProgram, FunctionRegistry, SBPFVersion},
    verifier::{RequisiteVerifier, Verifier},
};
use test_utils::{create_vm, TestContextObject};

use crate::common::ConfigTemplate;

mod common;

#[derive(arbitrary::Arbitrary, Debug)]
struct DumbFuzzData {
    template: ConfigTemplate,
    prog: Vec<u8>,
    mem: Vec<u8>,
}

fuzz_target!(|data: DumbFuzzData| {
    let prog = data.prog;
    let config = data.template.into();
    let function_registry = FunctionRegistry::default();
    let syscall_registry = FunctionRegistry::<BuiltinFunction<TestContextObject>>::default();

    if RequisiteVerifier::verify(&prog, &config, SBPFVersion::V3, &function_registry, &syscall_registry).is_err() {
        // verify please
        return;
    }
    let mut mem = data.mem;
    let executable = Executable::<TestContextObject>::from_text_bytes(
        &prog,
        std::sync::Arc::new(BuiltinProgram::new_loader(
            config,
        )),
        SBPFVersion::V3,
        function_registry,
    )
    .unwrap();
    let mem_region = MemoryRegion::new_writable(&mut mem, ebpf::MM_INPUT_START);
    let mut context_object = TestContextObject::new(29);
    create_vm!(
        interp_vm,
        &executable,
        &mut context_object,
        stack,
        heap,
        vec![mem_region],
        None
    );

    let (_interp_ins_count, interp_res) = interp_vm.execute_program(&executable, true);
    drop(black_box(interp_res));
});
