
macro_rules! include_check(($summary:expr, $check_name:expr, $desc:expr) =>
                           (if $summary.get_mut(&$check_name).is_none() {
                               $summary.insert($check_name,
                                               Status::new($desc));
                           }));

macro_rules! include_locators(($config:expr,
                               $status:expr,
                               $variable_name:expr,
                               $variable_index:expr,
                               $value_index:expr) =>
    (if let Some(include_locators) = $config.include_locators {
        if include_locators {
            let locator = Locator::new($variable_name.clone(),
                                       $variable_index,
                                       $value_index,
                                       None);
            if let Some(ref mut locators) = $status.locator {
                locators.insert(locator);
            } else {
                let mut set = HashSet::new();
                set.insert(locator);
                $status.locator = Some(set);
            }
        }
    }));

