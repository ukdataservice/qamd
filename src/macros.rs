
macro_rules! str_to_ptr(($name:expr) =>
                        (ok!(CString::new($name)).into_raw()));
macro_rules! ptr_to_str(($name:expr) =>
                        (CStr::from_ptr($name).to_string_lossy().into_owned()));
macro_rules! ok(($expression:expr) => ($expression.unwrap()));

