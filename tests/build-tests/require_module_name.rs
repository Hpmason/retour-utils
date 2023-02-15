use detour_lib_impl::hook_module;

#[hook_module]
mod lua {
    #[offset(0x1234)]
    fn lua_load() {

    }
}
// needed for trybuild
fn main() {}