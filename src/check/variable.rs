
use config::Config;
use report::{ Report, Variable, Status, Locator };
use check::{ contains, VariableCheckFn };

// Register the checks
pub fn register() -> Vec<VariableCheckFn> {
    vec!(variable_missing_label,
         date_format,
         variable_label_max_length,
         variable_odd_characters)
}

/// Variable checks

fn date_format(variable: &Variable,
               config: &Config,
               report: &mut Report) {
    // refer here for the docs on the date format. ReadStat internally
    // attempts to treat data as-if it were just Stata.
    // https://www.stata.com/help.cgi?datetime_display_formats

    if let Some(ref setting) = config
        .variable_config
        .date_format {
        include_check!(report.summary.date_format,
                       format!("{} {} {}",
                               "Flags date formats that are too",
                               "specific and could potentially",
                               "be disclosive.").as_str());
        let date_time_specifiers = &setting.setting;

        if let Some(ref mut status) = report.summary.date_format {
            if contains(&variable.value_format, &date_time_specifiers) {
                status.fail += 1;

                include_locators!(config,
                                  status,
                                  variable.name,
                                  variable.index,
                                  -1);
            } else {
                status.pass += 1;
            }
        }
    }
}

fn variable_missing_label(variable: &Variable,
                          config: &Config,
                          report: &mut Report) {
    if let Some(ref setting) = config
            .variable_config
            .missing_variable_labels {
        include_check!(report.summary.variable_label_missing,
                       "Variables should have a label.");

        if setting.setting {
            if let Some(ref mut status) = report
                .summary
                .variable_label_missing {

                if variable.label == "" {
                    status.fail += 1;

                    include_locators!(config,
                                      status,
                                      variable.name,
                                      variable.index,
                                      -1);
                } else {
                    status.pass += 1;
                }
            }
        }
    }
}

fn variable_label_max_length(variable: &Variable,
                             config: &Config,
                             report: &mut Report) {
    if let Some(ref setting) = config.variable_config.label_max_length {
        include_check!(report.summary.variable_label_max_length,
                       format!("{} ({} characters)",
                               "Variable labels cannot exceed a max length",
                               &setting.setting).as_str());

        if let Some(ref mut status) = report.summary.variable_label_max_length {
            if variable.label.len() > setting.setting as usize {
                status.fail += 1;

                include_locators!(config,
                                  status,
                                  variable.name,
                                  variable.index,
                                  -1);
            } else {
                status.pass += 1;
            }
        }
    }
}

fn variable_odd_characters(variable: &Variable,
                  config: &Config,
                  report: &mut Report) {
    if let Some(ref setting) = config.variable_config.odd_characters {
        include_check!(report.summary.variable_odd_characters,
                       format!("{} {} {:?}",
                               "Variable names and lables cannot contain",
                               "certain 'odd' characters. ",
                               setting.setting).as_str());

        if let Some(ref mut status) = report
            .summary
            .variable_odd_characters {
            if contains(&variable.name, &setting.setting) ||
                contains(&variable.label, &setting.setting) {
                status.fail += 1;

                include_locators!(config,
                                  status,
                                  variable.name,
                                  variable.index,
                                  -1);
            } else {
                status.pass += 1;
            }
        }
    }
}

