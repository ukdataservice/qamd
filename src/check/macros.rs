macro_rules! include_check(($summary:expr, $check_name:expr, $desc:expr, $category:expr) =>
                           (if $summary.get_mut(&$check_name).is_none() {
                               $summary.insert($check_name,
                                               Status::new($desc, $category));
                           }));

macro_rules! include_locators(($config:expr,
                               $status:expr,
                               $variable_name:expr,
                               $variable_index:expr,
                               $value_index:expr) =>
    (if let Some(metadata_only) = $config.metadata_only {
        if !metadata_only {
            let locator = Locator::new($variable_name.clone(),
                                       $variable_index,
                                       $value_index,
                                       None);
            if let Some(ref mut locators) = $status.locators {
                locators.insert(locator);
            } else {
                let mut set = HashSet::new();
                set.insert(locator);
                $status.locators = Some(set);
            }
        }
    }));
