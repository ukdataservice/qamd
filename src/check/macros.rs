
macro_rules! include_check(($expr:expr) =>
                           (if $expr.is_none() {
                               $expr = Some(Status::new());
                           }));
