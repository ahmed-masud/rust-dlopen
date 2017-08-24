use super::super::err::{Error};
use std::ffi::{CString, CStr, OsStr};

//choose the right platform implementation here
#[cfg(unix)]
use super::unix::{open_lib, get_sym, close_lib, Handle};
#[cfg(windows)]
use super::windows::{open_lib, get_sym, close_lib, Handle};

use std::mem::{transmute_copy, size_of};

/**
    Main interface for opening and working with a dynamic link library.

    **Note: ** Several methods have their "*_cstr" equivalents. This is because all native OS interfaces
    actually use C-strings. If you pass [`CStr'](https://doc.rust-lang.org/std/ffi/struct.CStr.html)
    as an argument, RawLib doesn't need to perform additional conversion from Rust string to
    C-string.. This makes `*_cstr" functions slightly more optimal than their normal equivalents.
    It is recommended that you use
    [const-cstr](https://github.com/abonander/const-cstr) crate to create statically allocated C-strings.

    **Note:** The handle to the library gets released when the library object gets dropped.
    Unless your application opened the library multiple times, this is the moment when symbols
    obtained from the library become dangling symbols.
*/
#[derive(Debug)]
pub struct RawLib {
    handle: Handle
}

impl RawLib {
    /**
    Opens a dynamic library.

    **Note:** different platforms search for libraries in different directories.
    Therefore this function cannot be 100% platform independent.
    However it seems that all platforms support the full path and
    searching in default os directories if you provide only the file name.
    Please refer to your operating system guide for precise information about the directories
    where the operating system searches for dynamic link libraries.

    #Example

    '''no_run
    mod dynlib;
    use dynlib::raw::RawLib;

    fn main() {
        //use full path
        let lib = DynLib::open("/lib/i386-linux-gnu/libm.so.6").unwrap();
        //use only file name
        let lib = DynLib::open("libm.so.6").unwrap();
    }
    ```
    */
    pub  fn open<S>(name: S) -> Result<RawLib, Error> where S: AsRef<OsStr> {
;
        Ok(Self {
            handle: unsafe{open_lib(name.as_ref())}?
        })
    }
    /**
    Obtains symbol from opened library.

    **Note:** the `T` template type needs to have a size of a pointer.
    Because Rust does not support static casts at the moment, the size of the type
    is checked in runtime and causes panic if it doesn't match.

    **Note:** It is legal for a library to export null symbols.
    However this is something that almost nobody expects.
    Therefore allowing it here would bring many problems, especially if user obtains references
    or functions.
    This method checks the address value and returns `Error::NullSymbol` error if the value is null.
    If your code does require obtaining symbols with null value, please do something like this:
    ```no_run
    extern crate dynlib;
    use dynlib::raw::RawLib;
    use dynlib::Error;
    use std::ptr::null;
    fn main(){
        let lib = RawLib::open("libyourlib.so").unwrap();
        let ptr_or_null: * const i32 = match lib.symbol("symbolname") {
            Ok(val) => val,
            Err(err) => match err {
                Error::NullSymbol => null(),
                _ => panic!("Could not obtain the symbol")
            }
        }
        //do something with the symbol
    }
    ```
    */
    pub unsafe fn symbol<T>(&self, name: &str) -> Result<T, Error> {
        let cname = CString::new(name)?;
        self.symbol_cstr(cname.as_ref())
    }
    ///Equivalent of the `symbol` method but takes `CStr` as a argument.
    pub unsafe fn symbol_cstr<T>(&self, name: &CStr) -> Result<T, Error> {
        //TODO: convert it to some kind of static assertion (not yet supported in Rust)
        //this comparison should be calculated by compiler at compilation time - zero cost
        if size_of::<T>() != size_of::<*mut ()>() {
            panic!("The type passed to dlopen::RawLib::symbol() function has a different size than a pointer - cannot transmute");
        }
        let raw = get_sym(self.handle, name)?;
        if raw.is_null() {
            return Err(Error::NullSymbol)
        } else {
            Ok(transmute_copy(&raw))
        }
    }
}

impl Drop for RawLib {
    fn drop(&mut self) {
        self.handle = close_lib(self.handle);
    }
}

unsafe impl Sync for RawLib {}
unsafe impl Send for RawLib {}