
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
                link(rel="stylesheet",
                     href="https://stackpath.bootstrapcdn.com/bootstrap/4.1.1/css/bootstrap.min.css",
                     integrity="sha384-WskhaSGFgHYWDcbwN70/dfYBj47jz9qbsMId/iRN3ewGhXQFZCSftd1LZCfmhktB",
                     crossorigin="anonymous");
            }
            body {
                div(class="container") {
                    div(class="row") {
                        h1(id="file-name", class="file-name") : &report.metadata.file_name;
                    }

                    br;

                    div(class="row") {
                        table(class="table table-bordered") {
                            tr {
                                th(scope="col") : "Name";
                                th(scope="col") : "Status";
                                th(scope="col") : "Description";
                            }

                            @ for check in report.summary.clone() {
                                @ if let (name, Some(status)) = check {
                                    @ if status.fail > 0 {
                                        tr(class="table-danger") {
                                            td(scope="row") : format!("{}", name);
                                            td : format!("failed ({})", status.fail);
                                            td : &status.desc;
                                        }
                                    } else {
                                        tr(class="table-success") {
                                            td(scope="row") : format!("{}", name);
                                            td : "passed";
                                            td : &status.desc;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

