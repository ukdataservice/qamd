
use horrorshow::helper::doctype;

use report::Report;

pub fn to_html(report: &Report) -> String {
    format!("{}", html!{
        : doctype::HTML;
        html {
            head {
                title : &report.metadata.file_name;
                style : "table, th, td { border: 1px solid black; padding: 4px; }";
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
                            tr {
                                td { : format!("{}", name) }
                                td { : "passed/failed" }
                                td { : "this is a test demonstrating the format " }
                            }
                        }
                    }
                }
            }
        }
    })
}

