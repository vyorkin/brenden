// The link attribute tells `rustc` that we need to link these functions to zlib.
//
// This is equivalent to adding `-lz` flag at link time.
//
// Can verify this by running:
// on macOS: `otool -L target/debug/zlib-wrapper`,
// on Linux: `ldd`
// on Windows: `dumpbin`

use libc::{c_int, c_ulong};
#[link(name = "z")]
unsafe extern "C" {
    // --------------------------------------------------------------------------
    // Definitions from zlib.h
    // --------------------------------------------------------------------------
    //
    // #ifndef ZEXTERN
    // #  define ZEXTERN extern
    // #endif
    // #ifndef ZEXPORT
    // #  define ZEXPORT
    // #endif
    //
    // typedef unsigned char  Byte;  /* 8 bits */
    // typedef unsigned int   uInt;  /* 16 bits or more */
    // typedef unsigned long  uLong; /* 32 bits or more */
    //
    // typedef Byte  FAR Bytef;
    // typedef char  FAR charf;
    // typedef int   FAR intf;
    // typedef uInt  FAR uIntf;
    // typedef uLong FAR uLongf;
    //
    // ZEXTERN int ZEXPORT compress OF((Bytef *dest,   uLongf *destLen,
    //                                  const Bytef *source, uLong sourceLen));
    //
    // ZEXTERN uLong ZEXPORT compressBound OF((uLong sourceLen));
    //
    // ZEXTERN int ZEXPORT uncompress OF((Bytef *dest,   uLongf *destLen,
    //                                    const Bytef *source, uLong sourceLen));
    // --------------------------------------------------------------------------

    unsafe fn compress(
        dest: *mut u8,
        dest_len: *mut c_ulong,
        source: *const u8,
        source_len: c_ulong,
    ) -> c_int;

    // Estimates the size of buffer required to
    // compress `source_len` bytes of data using the compress()
    unsafe fn compressBound(source_len: c_ulong) -> c_ulong;

    unsafe fn uncompress(
        dest: *mut u8,
        dest_len: *mut c_ulong,
        source: *const u8,
        source_len: c_ulong,
    ) -> c_int;
}

pub fn zlip_compress(source: &[u8]) -> Vec<u8> {
    unsafe {
        let source_len = source.len() as c_ulong;

        let mut dest_len = compressBound(source_len);
        let mut dest = Vec::with_capacity(dest_len as usize);

        compress(
            dest.as_mut_ptr(),
            &mut dest_len,
            source.as_ptr(),
            source_len,
        );
        dest.set_len(dest_len as usize);
        dest
    }
}

pub fn zlib_uncompress(source: &[u8], max_dest_len: usize) -> Vec<u8> {
    unsafe {
        let source_len = source.len() as c_ulong;

        let mut dest_len = max_dest_len as c_ulong;
        let mut dest = Vec::with_capacity(max_dest_len);

        uncompress(
            dest.as_mut_ptr(),
            &mut dest_len,
            source.as_ptr(),
            source_len,
        );

        dest.set_len(dest_len as usize);
        dest
    }
}

fn main() {
    let hello_zlib = "hello, zlib, no exclamation mark".as_bytes();
    let hello_zlib_compressed = zlip_compress(hello_zlib);
    let hello_zlib_uncompressed = zlib_uncompress(&hello_zlib_compressed, 100);

    assert_eq!(hello_zlib, hello_zlib_uncompressed);

    let hello_zlib_utf8 = String::from_utf8(hello_zlib_uncompressed).expect("Invalid characters");
    println!("{}", hello_zlib_utf8);
}
