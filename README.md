# retour-utils
This crate is meant to help creating detours with the [retour crate](https://crates.io/crates/retour). If you're creating *lots* of detours, it's very repetitive, so this crate adds some a few helper functions and macro to greatly simplify/streamline the process. It works on both Unix and Windows.



## Example

```rust
use retour_utils::hook_module;

#[hook_module("lua52.dll")]
mod lua {
    // #[hook_module] will create this
    // const MODULE_NAME: &str = "lua52.dll"
    // and
    // pub unsafe init_detours() -> crate::Result<()> {..}
    // which will initialize all the StaticDetours generated by the macro inside this module

    #[allow(non_camel_case_types)]
    type lua_State = ();
    #[allow(non_camel_case_types)]
    type lua_Alloc = ();
    
    // Creates a StaticDetour called Lua_newstate with the same function type as our function 
    // (minus abi/unsafe to work with retour crate)
    #[hook(unsafe extern "C" Lua_newstate, symbol = "Lua_newstate")]
    pub fn newstate(f: *mut lua_Alloc, ud: *mut std::ffi::c_void) -> *mut lua_State {
        unsafe {
            Lua_newstate.call(f, ud)
        }
    }
    // More lua hooks
}


// #[hook_module] creates a `init_hooks` function that initializes and enables all the hooks
lua::init_hooks().unwrap()
```

This is very much in the early stages, with some noticable rough areas
- No docs yet
- Naming of macros and fns likely to change for consistency/clarity
