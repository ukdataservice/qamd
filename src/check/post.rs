
use readstat::context::Context;
use config::Config;
use report::{ Report, Status, Locator };
use report::missing::Missing;
use check::{ PostCheckFn, contains };

/// Returns a vec of the functions provided by this module
pub fn register() -> Vec<PostCheckFn> {
    vec!(primary_variable,
         date_variables,
         system_missing_over_threshold,
         variables_with_unique_values)
}

fn date_variables(context: &Context,
                  _config: &Config,
                  _report: &mut Report) {
    // refer here for the docs on the date format. ReadStat internally
    // attempts to treat data as-if it were just Stata.
    // https://www.stata.com/help.cgi?datetime_display_formats

    for variable in context.variables.iter() {
        // println!("{:#?}", variable.value_format);
        let date_time_specifiers = vec!(
            // date
            "JJJ",     // Day within year (001 - 366)
            "jjj",     // Day within year (1 - 366)
            "DD",      // Day within month (01 - 31)
            "dd",      // Day within month (1 - 31)
            "DAYNAME", // day of week        Sunday, Monday, ... (aligned)
            "Dayname", // day of week        Sunday, Monday, ... (unaligned)
            "Day",     // day of week        Sun, Mon, ...
            "Da",      // day of week        Su, Mo, ...
            "day",     // day of week        sun, mon, ...
            "da",      //day of week        su, mo, ...
            // time
            "HH",   // hour               00 - 23
            "Hh",   // hour               00 - 12
            "hH",   // hour               0 - 23
            "hh",   // hour               0 - 12
            "MM",   // minute             00 - 59
            "mm",   // minute             0 - 59
            "SS",   // second             00 - 60 (sic, due to leap seconds)
            "ss",   // second             0 - 60 (sic, due to leap seconds)
            ".s",   // tenths             .0 - .9
            ".ss",  // hundredths         .00 - .99
            ".sss", // thousandths        .000 - .999
            "Y",
            "y",
        ).iter().map(|s| s.to_string()).collect();

        if contains(&variable.value_format, &date_time_specifiers) {
            println!("variable {} is a date! : {}",
                     variable.name,
                     variable.value_format);
        }
    }
}

/// Count the number of cases using the provided primary variable_count
fn primary_variable(context: &Context,
                    config: &Config,
                    report: &mut Report) {
    if let Some(ref primary_variable) = config.primary_variable {
        if report.metadata.case_count.is_none() {
            report.metadata.case_count = Some(0);
        }

        if let Some((_variable, map)) = context.frequency_table
            .iter().find(|(variable, _)| {
            variable.name == primary_variable.setting
        }) {
            // report count of distinct cases for this variable
            report.metadata.case_count = Some(map.keys().len() as i32);
        }
    }
}

/// Report variables with a number of system missing values over a
/// specified threhold.
fn system_missing_over_threshold(context: &Context,
                                 config: &Config,
                                 report: &mut Report) {

    //println!("frequency_table: {:#?}", context.frequency_table);

    if let Some(ref setting) = config
            .value_config
            .system_missing_value_threshold {
        include_check!(report.summary.system_missing_over_threshold,
                       format!("{} {} (Threshold: {}%)",
                               "Variables with large quantities of",
                               "values missing.",
                               setting.setting).as_str());

        if let Some(ref mut status) = report
                .summary
                .system_missing_over_threshold {
            // map between variable and % missing

            // pull count of sysmiss values from Context.frequency_table
            // sum to percentage of sysmiss (delivered as NaN)

            for (variable, map) in &context.frequency_table {

                let sum = map.iter().fold(0, |mut sum, (_, occ)| {
                    sum += occ;
                    sum
                });

                assert_eq!(report.metadata.raw_case_count, sum);

                // compare with config threhold
                // and increment pass/fail
                if let Some((_, count)) = map
                    .iter().find(|(value, _)| {
                        value.missing == Missing::SYSTEM_MISSING
                    }) {
                    let sys_miss = (*count as f32 / sum as f32) * 100.0;
                    if sys_miss > setting.setting as f32 {
                        status.fail += 1;

                        include_locators!(config, status, variable.index, -1);
                    }
                }
            }

            status.pass = report.metadata.variable_count - status.fail;

        }
    }
}

/// Count the number of variables with one or more unique values
fn variables_with_unique_values(context: &Context,
                                config: &Config,
                                report: &mut Report) {
    if let Some(ref setting) = config.variables_with_unique_values {
        include_check!(report.summary.variables_with_unique_values,
                       "Detects values as outliers if they unique.");

        if let Some(ref mut status) = report.summary
            .variables_with_unique_values {
            for (variable, map) in context.frequency_table.iter() {
                if let Some(_) = map.iter().find(|(_value, occ)| {
                    *occ <= &setting.setting
                }) {
                    status.fail += 1;

                    include_locators!(config, status, variable.index, -1);
                } else {
                    status.pass += 1
                }
            }
        }
    }
}

