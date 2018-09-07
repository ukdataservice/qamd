macro_rules! str_to_ptr(($name:expr) =>
                        (ok!(CString::new($name)).into_raw()));
macro_rules! ptr_to_str(($name:expr) =>
                        (CStr::from_ptr($name).to_string_lossy().into_owned()));
macro_rules! ok(($expression:expr) => ($expression.unwrap()));

#[allow(unused_macros)]
macro_rules! debug {
    ($text:expr) => ( println!("[{}:{}:{}] {}",
                               file!(),
                               line!(),
                               column!(),
                               $text);
                    );
    ($text:expr, $($args:expr)*) => ( println!("[{}:{}:{}] {}",
                                               file!(),
                                               line!(),
                                               column!(),
                                               format_args!($text, $($args)*));
                                    );
}

#[cfg(test)]
macro_rules! assert_setting {
    ($setting:expr, $pass:expr, $fail:expr) => {
        if let Some(ref status) = $setting {
            assert_eq!(status.pass, $pass);
            assert_eq!(status.fail, $fail);
        } else {
            assert!(false, " is None and should be Some(Setting)");
        }
    };
}
