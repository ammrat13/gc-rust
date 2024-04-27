use std::arch::asm;

#[cfg(not(target_arch = "x86_64"))]
compile_error!("Unsupported architecture");

#[inline(always)]
pub fn get_stack_pointer() -> usize {
    // Make sure to clobber all the caller-save registers to make sure they make
    // it onto the stack.
    let ret: usize;
    unsafe {
        if cfg!(target_arch = "x86_64") {
            asm!(
                "mov {ret}, rsp",
                ret = out(reg) ret,
                out("r12") _,
                out("r13") _,
                out("r14") _,
                out("r15") _,
            );
        } else {
            unreachable!();
        }
    }
    ret
}

pub fn get_pointer_size() -> usize {
    if cfg!(target_pointer_width = "64") {
        8
    } else {
        4
    }
}
