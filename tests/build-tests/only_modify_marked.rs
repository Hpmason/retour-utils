use detour_lib_impl::hook_module;

#[hook_module]
mod lua {
    pub fn left_alone(foo: i32) -> i32 {
        foo
    }

    pub const DATA1: usize = 4;
    pub static DATA2: usize = 2;
}
// needed for trybuild
fn main() {
    use lua::*;
    // won't run, but will verify data types are kept consistent
    assert_eq!(DATA1, 4 as usize);
    assert_eq!(DATA2, 2 as usize);
    assert_eq!(left_alone(0), 0 as i32);
}