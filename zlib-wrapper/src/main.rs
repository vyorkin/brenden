use std::ffi::CString;

use libc::{c_char, c_int, c_uchar, c_uint, c_ulong};

// The link attribute tells `rustc` that we need to link these functions to zlib.
//
// This is equivalent to adding `-lz` flag at link time.
// Instructs rustc that these functions belong to the external "z" library.
//
// Can verify this by running:
// on macOS: `otool -L target/debug/zlib-wrapper`,
// on Linux: `ldd`
// on Windows: `dumpbin`
#[link(name = "z")]
unsafe extern "C" {
    // The biggest challenge is mapping verious types and functions from C to Rust + libc crate.
    // The rust-bindgen tool can generate bindings to C libraries automatically from C headers.
    //
    // But sometimes dealing with rust-bindgen is not worth the trouble for simple cases.
    // In those cases we can:
    // 1. Copy the C struct definition.
    // 2. Convert the C types to Rust types.
    // 3. Implement function interfaaces.

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

// Align memory in this struct as C compiler would.
// A C struct representing zlib file state, as defined in zlib.h
#[repr(C)]
struct GzFileState {
    have: c_uint,
    next: *mut c_uchar,
    pos: i64,
}

type GzFile = *mut GzFileState;

// Instructs rustc that these functions belong to the external "z" library.
// Yes, "z" in the name of the "zlib" library.
#[link(name = "z")]
unsafe extern "C" {
    // struct gzFile_s {
    //     unsigned have;
    //     unsigned char *next;
    //     z_off64_t pos;
    // };
    //
    // typedef struct gzFile_s *gzFile;
    //
    // ZEXTERN gzFile ZEXPORT gzopen OF((const char *, const char *));
    // ZEXTERN int ZEXPORT gzread OF((gzFile file, voidp buf, unsigned len));
    // ZEXTERN int ZEXPORT gzclose OF((gzFile file));
    // ZEXTERN int ZEXPORT gzeof OF((gzFile file));

    unsafe fn gzopen(path: *const c_char, mode: *const c_char) -> GzFile;
    unsafe fn gzread(file: GzFile, buf: *mut c_uchar, len: c_uint) -> c_int;
    unsafe fn gzclose(file: GzFile) -> c_int;
    unsafe fn gzeof(file: GzFile) -> c_int;
}

// Opens gzipped file, reads its contents, and returns them as a string.
fn read_gz_file(name: &str) -> String {
    let mut buffer = [0u8; 0x1000]; // 16^3 = 4096 bytes
    let mut contents = String::new();
    unsafe {
        // Convert rust UTF-8 into an ASCII C-string
        let c_name = CString::new(name).expect("CString failed");
        let c_mode = CString::new("r").expect("CString failed");
        let file = gzopen(c_name.as_ptr(), c_mode.as_ptr());
        if file.is_null() {
            panic!("Couldn't read file: {}", std::io::Error::last_os_error());
        }
        while gzeof(file) == 0 {
            let bytes_read = gzread(file, buffer.as_mut_ptr(), (buffer.len() - 1) as c_uint);
            let s = std::str::from_utf8(&buffer[..(bytes_read as usize)]).unwrap();
            contents.push_str(s);
        }
        gzclose(file);

        contents
    }
}

fn main() {
    println!("1. compress/decompress");

    let hello_zlib = "hello, zlib, no exclamation mark".as_bytes();
    let hello_zlib_compressed = zlip_compress(hello_zlib);
    let hello_zlib_uncompressed = zlib_uncompress(&hello_zlib_compressed, 100);

    assert_eq!(hello_zlib, hello_zlib_uncompressed);

    let hello_zlib_utf8 = String::from_utf8(hello_zlib_uncompressed).expect("Invalid characters");
    println!("{}", hello_zlib_utf8);

    println!("1. read_gz_file");

    println!("{}", read_gz_file("file.txt.gz"));
}
