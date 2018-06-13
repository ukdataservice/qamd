
use config::Config;
use report::{ Report, Status, Variable };
use check::{ contains, VariableCheckFn };

// Register the checks
pub fn register() -> Vec<VariableCheckFn> {
    vec!(variable_missing_label,
         variable_label_max_length,
         variable_odd_characters)
}

/// Variable checks

fn variable_missing_label(variable: &Variable,
                          config: &Config,
                          report: &mut Report) {
    include_check!(report.summary.variable_label_missing);

    if let Some(ref setting) = config
            .variable_config
            .missing_variable_labels {
        if setting.setting {
            if let Some(ref mut status) = report
                .summary
                .variable_label_missing {

                if variable.label == "" {
                    status.fail += 1;
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
    include_check!(report.summary.variable_label_max_length);

    if let Some(ref setting) = config.variable_config.label_max_length {
        if let Some(ref mut status) = report.summary.variable_label_max_length {
            if variable.label.len() > setting.setting as usize {
                status.fail += 1;
            } else {
                status.pass += 1;
            }
        }
    }
}

fn variable_odd_characters(variable: &Variable,
                  config: &Config,
                  report: &mut Report) {
    include_check!(report.summary.variable_odd_characters);

    if let Some(ref setting) = config.variable_config.odd_characters {
        if let Some(ref mut status) = report
            .summary
            .variable_odd_characters {
            if contains(&variable.name, &setting.setting) ||
                contains(&variable.label, &setting.setting) {
                status.fail += 1;
            } else {
                status.pass += 1;
            }
        }
    }
}

