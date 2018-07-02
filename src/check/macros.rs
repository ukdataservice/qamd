
macro_rules! include_check(($setting:expr, $desc:expr) =>
                           (if $setting.is_none() {
                               $setting = Some(Status::new($desc));
                           }));

