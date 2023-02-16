use retour_utils_impl::hook_module;

#[hook_module("")]
mod lua {
    use retour::StaticDetour;
    fn hooky() -> i32 {
        0
    }
    // #[offset(0xBEEF)]
    // pub fn hooky(detour: StaticDetour<fn() -> i32>) -> i32 {
    //     0
    // }

    // pub fn left_alone(foo: i32) -> i32 {
    //     foo
    // }

    // pub const DATA1: usize = 4;
    // pub static DATA2: usize = 2;
}
// needed for trybuild
fn main() {
    use lua::*;
}
