


pub mod error;

use std::iter;

pub use retour_utils_impl::hook_module;
pub use error::Error;
use windows::{Win32::{Foundation::HINSTANCE, System::LibraryLoader::GetModuleHandleW}, core::PCWSTR};

type Result<T> = std::result::Result<T, error::Error>;

pub enum LookupData {
    Offset {
        module: &'static str,
        offset: usize,
    },
    Symbol {
        module: &'static str,
        symbol: &'static str,
    }
}

impl LookupData {
    pub const fn from_offset(module: &'static str, offset: usize) -> Self {
        Self::Offset { module, offset }
    }

    pub const fn from_symbol(module: &'static str, symbol: &'static str) -> Self {
        Self::Symbol { module, symbol }
    }
    fn get_module(&self) -> &str {
        match self {
            Self::Offset { module, .. } => module,
            Self::Symbol { module, .. } => module,
        }
    }
    #[cfg(windows)]
    fn address_from_handle(&self, handle: HINSTANCE) -> Option<*const ()> {
        use std::ffi::CString;

        use windows::{Win32::System::LibraryLoader::GetProcAddress, core::PCSTR};

        match self {
            LookupData::Offset { offset, ..} => {
                // On Windows, HINSTANCE is the start address of the library, 
                //  so we just add the offset to get the address 
                Some((handle.0 as usize + offset) as *const ())
            },
            LookupData::Symbol { symbol, .. } => {
                let c_symbol = CString::new(symbol.clone()).ok()?;
                let wrapped_ptr = PCSTR::from_raw(c_symbol.as_ptr() as *const u8);
                if let Some(func_ptr) = unsafe { GetProcAddress(handle, wrapped_ptr) } {
                    Some(func_ptr as *const ())
                } else {
                    None
                }
            },
        }
    } 
}

pub unsafe fn init_detour(lookup_data: LookupData, init_detour: fn(*const ()) -> retour::Result<()>) -> Result<()> {
    let module = lookup_data.get_module().to_string();
    let module_w_ptr = module
        .encode_utf16()
        .chain(iter::once(0))
        .collect::<Vec<u16>>()
        .as_ptr();
    let wrapped_ptr = PCWSTR::from_raw(module_w_ptr);

    // Get handle to module (aka dll)
    if let Ok(handle) = unsafe { GetModuleHandleW(wrapped_ptr) } {
        let Some(addr) = lookup_data.address_from_handle(handle) else {
            return Err(Error::ModuleNotLoaded);
        };
        init_detour(addr)?;
    }
    Ok(())
}