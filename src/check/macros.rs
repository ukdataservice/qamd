
macro_rules! include_check(($setting:expr, $desc:expr) =>
                           (if $setting.is_none() {
                               $setting = Some(Status::new($desc));
                           }));

macro_rules! include_locators(($config:expr,
                               $status:expr,
                               $variable_index:expr,
                               $value_index:expr) =>
    (if let Some(include_locators) = $config.include_locators {
        if include_locators {
            let locator = Locator::new($variable_index, $value_index);
            if let Some(ref mut locators) = $status.locator {
                locators.push(locator);
            } else {
                $status.locator = Some(vec!(locator));
            }
        }
    }));

