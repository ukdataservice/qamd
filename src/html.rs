
// use horrorshow::prelude::*;
use horrorshow::helper::doctype;

use report::Report;

pub fn to_html(report: &Report) -> String {
    format!("{}", html!{
        : doctype::HTML;
        html {
            head {
                title : &report.metadata.file_name;
                meta(charset="UTF-8");
                style : r#"
                    table, th, td {
                        border: 1px solid black;
                        padding: 4px;
                    }
                    .passed {
                        background-color: rgb(212, 237, 218);;
                    }
                    .failed {
                        background-color: rgb(248, 215, 218);;
                    }
                    "#;
            }
            body {
                h1(id="file-name", class="file-name") : &report.metadata.file_name;

                table {
                    tr {
                        th { p { : "Name" } }
                        th { p { : "Status" } }
                        th { p { : "Description" } }
                    }

                    @ for check in report.summary.clone() {
                        @ if let (name, Some(status)) = check {
                            @ if status.fail > 0 {
                                tr(class="failed") {
                                    td { : format!("{}", name) }
                                    td : format!("failed ({})", status.fail);
                                    td : &status.desc;
                                }
                            } else {
                                tr(class="passed") {
                                    td { : format!("{}", name) }
                                    td : "passed";
                                    td : &status.desc;
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

