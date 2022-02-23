use mitosis_macros::*;

declare_module_param!(sample_long, u64);
declare_module_param!(sample_int, u32);
declare_module_param!(sample_str, *mut u8);
