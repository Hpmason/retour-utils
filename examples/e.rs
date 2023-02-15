
use hooking_macro_impl::hook_module;

#[hook_module("Apple.dll")]
mod lua {
    #[offset(0x500)]
    fn lua_newstate() {

    }

    fn normal_fn() {
        
    }
}
fn main() {}