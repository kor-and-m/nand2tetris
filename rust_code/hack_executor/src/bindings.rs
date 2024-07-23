#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct HackExecutor {
    pub a: ::std::os::raw::c_short,
    pub d: ::std::os::raw::c_short,
    pub pc: ::std::os::raw::c_short,
    pub memory: *mut ::std::os::raw::c_short,
    pub program: *mut ::std::os::raw::c_short,
}

impl HackExecutor {
    pub fn new(instructions: &mut [i16]) -> *mut Self {
        unsafe { init_hack_executor(instructions.as_mut_ptr()) }
    }

    pub fn run(s: *mut Self, iteration_count: usize) {
        unsafe { run_executor(s, iteration_count as i16) }
    }

    pub fn result(s: *mut Self) -> i16 {
        unsafe { read_stack_value(s) }
    }

    pub fn read_memory(s: *mut Self, p: i16) -> i16 {
        unsafe { read_memory(s, p) }
    }

    pub fn drop(s: *mut Self) {
        unsafe { free_hack_executor(s) }
    }
}

extern "C" {
    fn init_hack_executor(instructions: *mut ::std::os::raw::c_short) -> *mut HackExecutor;
    fn free_hack_executor(executor: *mut HackExecutor);
    fn run_executor(executor: *mut HackExecutor, iterations: ::std::os::raw::c_short);
    fn read_memory(
        executor: *mut HackExecutor,
        pointer: ::std::os::raw::c_short,
    ) -> ::std::os::raw::c_short;
    fn read_stack_value(executor: *mut HackExecutor) -> ::std::os::raw::c_short;
}
