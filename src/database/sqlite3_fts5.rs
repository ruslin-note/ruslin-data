// https://gist.github.com/ColonelThirtyTwo/3dd1fe04e4cff0502fa70d12f3a6e72e

use std::{
    convert::TryFrom,
    ffi::{c_void, CStr, CString},
    os::raw::{c_char, c_int},
    panic::{catch_unwind, AssertUnwindSafe},
};

use diesel::sqlite::SqliteConnection;
// use diesel::result::Error
use diesel::libsqlite3_sys as ffi;
use ffi::{SQLITE_ERROR, SQLITE_OK};

pub struct SqliteError;

/// Combined so we can pattern match on it
const FTS5_TOKENIZE_QUERY_PREFIX: c_int = ffi::FTS5_TOKENIZE_QUERY | ffi::FTS5_TOKENIZE_PREFIX;

/// Reason the tokenizer is being called
pub enum TokenizeReason {
    /// Document is being inseted or removed
    Document,
    /// Running a MATCH query.
    Query {
        /// Whether this is a prefix query. If so, the last token emitted will be treated as a token prefix.
        prefix: bool,
    },
    /// Manually invoked via `fts5_api.xTokenize`.
    Aux,
}
impl TokenizeReason {
    fn from_const(v: c_int) -> Option<Self> {
        let v = match v {
            ffi::FTS5_TOKENIZE_DOCUMENT => Self::Document,
            ffi::FTS5_TOKENIZE_QUERY => Self::Query { prefix: false },
            FTS5_TOKENIZE_QUERY_PREFIX => Self::Query { prefix: true },
            ffi::FTS5_TOKENIZE_AUX => Self::Aux,
            _ => return None,
        };
        Some(v)
    }
}

/// Tokenizer implementation
pub trait Tokenizer: Sized + Send + 'static {
    /// Global data available to the `new` function
    type Global: Send + 'static;
    /// Creates a new instance of the tokenizer
    fn new(global: &Self::Global, args: Vec<String>) -> Result<Self, SqliteError>;
    /// Tokenizes a string.
    /// Should inspect the `text` object and call the `push_token` callback for each token.
    /// The callback takes 3 arguments: the token, the location within the `text` the token appears in,
    /// and a boolean flag that corresponds to the `FTS5_TOKEN_COLOCATED` flag.
    fn tokenize<TKF>(
        &mut self,
        reason: TokenizeReason,
        text: &[u8],
        push_token: TKF,
    ) -> Result<(), SqliteError>
    where
        TKF: FnMut(&[u8], std::ops::Range<usize>, bool) -> Result<(), SqliteError>;
}

unsafe extern "C" fn c_xcreate<T: Tokenizer>(
    global: *mut c_void,
    args: *mut *const c_char,
    nargs: c_int,
    out_tok: *mut *mut ffi::Fts5Tokenizer,
) -> c_int {
    let global = &*global.cast::<T::Global>();
    let args = (0..nargs as usize)
        .map(|i| *args.add(i))
        .map(|s| CStr::from_ptr(s).to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let res = catch_unwind(AssertUnwindSafe(move || T::new(global, args)));
    match res {
        Ok(Ok(v)) => {
            let bp = Box::into_raw(Box::new(v));
            *out_tok = bp.cast::<ffi::Fts5Tokenizer>();
            SQLITE_OK
        }
        // Ok(Err(rusqlite::Error::SqliteFailure(e, _))) => e.extended_code, TODO
        Ok(Err(_)) => SQLITE_ERROR,
        Err(msg) => {
            log::error!(
                "<{} as Tokenizer>::new paniced: {}",
                std::any::type_name::<T>(),
                panic_err_to_str(&msg)
            );
            SQLITE_ERROR
        }
    }
}

unsafe extern "C" fn c_xdelete<T: Tokenizer>(v: *mut ffi::Fts5Tokenizer) {
    let b = Box::from_raw(v.cast::<T>());
    match catch_unwind(AssertUnwindSafe(move || std::mem::drop(b))) {
        Ok(()) => {}
        Err(e) => {
            log::error!(
                "{}::drop paniced: {}",
                std::any::type_name::<T>(),
                panic_err_to_str(&e)
            );
        }
    }
}

unsafe extern "C" fn c_xdestroy<T: Tokenizer>(v: *mut c_void) {
    let b = Box::from_raw(v.cast::<T::Global>());
    match catch_unwind(AssertUnwindSafe(move || std::mem::drop(b))) {
        Ok(()) => {}
        Err(e) => {
            log::error!(
                "{}::drop paniced: {}",
                std::any::type_name::<T::Global>(),
                panic_err_to_str(&e)
            );
        }
    }
}

unsafe extern "C" fn c_xtokenize<T: Tokenizer>(
    this: *mut ffi::Fts5Tokenizer,
    ctx: *mut c_void,
    flags: c_int,
    data: *const c_char,
    data_len: c_int,
    push_token: Option<
        unsafe extern "C" fn(*mut c_void, c_int, *const c_char, c_int, c_int, c_int) -> c_int,
    >,
) -> c_int {
    let this = &mut *this.cast::<T>();
    let reason = match TokenizeReason::from_const(flags) {
        Some(v) => v,
        None => {
            log::error!("Unrecognized flags passed to xTokenize: {}", flags);
            return SQLITE_ERROR;
        }
    };

    let data = std::slice::from_raw_parts(data.cast::<u8>(), data_len as usize);

    let push_token = push_token.unwrap();
    let push_token =
        |token: &[u8], range: std::ops::Range<usize>, colocated: bool| -> Result<(), SqliteError> {
            let ntoken = c_int::try_from(token.len()).expect("Token length is took long");
            assert!(
                range.start <= data.len() && range.end <= data.len(),
                "Token range is invalid. Range is {:?}, data length is {}",
                range,
                data.len(),
            );
            let start = range.start as c_int;
            let end = range.end as c_int;
            let flags = if colocated {
                ffi::FTS5_TOKEN_COLOCATED
            } else {
                0
            };

            let res = (push_token)(
                ctx,
                flags,
                token.as_ptr().cast::<c_char>(),
                ntoken,
                start,
                end,
            );
            if res == SQLITE_OK {
                Ok(())
            } else {
                todo!()
                // Err(rusqlite::Error::SqliteFailure(
                // 	rusqlite::ffi::Error::new(res),
                // 	None,
                // ))
            }
        };

    match catch_unwind(AssertUnwindSafe(|| this.tokenize(reason, data, push_token))) {
        Ok(Ok(())) => SQLITE_OK,
        // Ok(Err(rusqlite::Error::SqliteFailure(e, _))) => e.extended_code, TODO
        Ok(Err(_)) => SQLITE_ERROR,
        Err(msg) => {
            log::error!(
                "<{} as Tokenizer>::tokenize paniced: {}",
                std::any::type_name::<T>(),
                panic_err_to_str(&msg)
            );
            SQLITE_ERROR
        }
    }
}

fn panic_err_to_str(msg: &Box<dyn std::any::Any + Send>) -> &str {
    if let Some(msg) = msg.downcast_ref::<String>() {
        msg.as_str()
    } else if let Some(msg) = msg.downcast_ref::<&'static str>() {
        *msg
    } else {
        "<non-string panic reason>"
    }
}

pub fn register_tokenizer<T: Tokenizer>(
    conn: &mut SqliteConnection,
    global_data: T::Global,
    name: &str,
) -> Result<(), String> {
    unsafe {
        let dbp = conn.get_raw_conn();
        let mut api: *mut ffi::fts5_api = std::ptr::null_mut();
        let mut stmt: *mut ffi::sqlite3_stmt = std::ptr::null_mut();

        let q = "SELECT fts5(?1)";
        if ffi::sqlite3_prepare(
            dbp,
            q.as_ptr().cast::<c_char>(),
            q.len() as c_int,
            &mut stmt,
            std::ptr::null_mut(),
        ) != SQLITE_OK
        {
            return Err("sqlite3_prepare failed".into());
        }
        ffi::sqlite3_bind_pointer(
            stmt,
            1,
            (&mut api) as *mut _ as *mut c_void,
            "fts5_api_ptr\0".as_ptr().cast::<c_char>(),
            None,
        );
        ffi::sqlite3_step(stmt);
        ffi::sqlite3_finalize(stmt);

        if api.is_null() {
            return Err("Could not get fts5 api".into());
        }

        let name = CString::new(name).map_err(|_| "Name has a null character in it")?;
        let global_data = Box::into_raw(Box::new(global_data));

        let e = ((*api).xCreateTokenizer.as_ref().unwrap())(
            api,
            name.as_ptr(),
            global_data.cast::<c_void>(),
            &mut ffi::fts5_tokenizer {
                xCreate: Some(c_xcreate::<T>),
                xDelete: Some(c_xdelete::<T>),
                xTokenize: Some(c_xtokenize::<T>),
            },
            Some(c_xdestroy::<T>),
        );
        if e != SQLITE_OK {
            return Err("xCreateTokenizer failed".into());
        }
        Ok(())
    }
}
