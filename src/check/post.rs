
use Context;
use config::Config;
use report::{ Report, Status };
use report::missing::Missing;
use check::PostCheckFn;

/// Returns a vec of the functions provided by this module
pub fn register() -> Vec<PostCheckFn> {
    vec!(system_missing_over_threshold)
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
        include_check!(report.summary.system_missing_over_threshold);

        if let Some(ref mut status) = report
                .summary
                .system_missing_over_threshold {
            // map between variable and % missing

            // pull count of sysmiss values from Context.frequency_table
            // sum to percentage of sysmiss (delivered as NaN)
            //
            //

            for (_variable, map) in &context.frequency_table {

                let sum = map.iter().fold(0, |mut sum, (_, occ)| {
                    sum += occ;
                    sum
                });

                assert_eq!(report.metadata.raw_case_count, sum);

                // compare with config threhold
                // and increment pass/fail
                if let Some((_, count)) = map.iter().find(|(value, _)| value.missing == Missing::SYSTEM_MISSING) {
                    let sys_miss = (*count as f32 / sum as f32) * 100.0;
                    if sys_miss > setting.setting as f32 {
                        status.fail += 1;
                    }
                }
            }

            status.pass = report.metadata.variable_count - status.fail;

        }
    }
}

