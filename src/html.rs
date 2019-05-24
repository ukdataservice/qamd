use horrorshow::helper::doctype;

use horrorshow::{Render, RenderBox};

use report::{Locator, Report};

use std::collections::HashSet;

pub fn to_html(report: &Report) -> String {
    let javascript = r#"
'use strict';

$(function() {

  $('tr.table-danger').click(function(obj) {
    var name = obj.currentTarget.children[0].innerText

    $('h2#selected-check').first().removeClass('d-none');
    $('h2#selected-check')[0].innerText = name;

    var selector = 'table#' + name.toLowerCase().replace(/ /g, '_');

    // hide all the tables
    $('table.table.table-striped').each(function(index, elem) {
      $('table#' + elem.id).addClass('d-none');
    });

    // show the selected table
    $(selector).first().removeClass('d-none');
  });
});
"#;

    format!(
        "{}",
        html! {
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
                        : logo();

                        div(id="title", class="row") {
                            h1(id="file-name") : &report.metadata.file_name;
                        }

                        div(class="row metadata") {
                            strong : format!("Raw Case Count: {}",
                                      report.metadata.raw_case_count);
                        }

                        div(class="row metadata") {
                            @ if let Some(case_count) = report.metadata.case_count {
                                strong : format!("Aggregated Case Count: {}",
                                          case_count);
                            }
                        }

                        div(class="row metadata") {
                            strong : format!("Total Variables: {}",
                                      report.metadata.variable_count);
                        }

                        div(class="row metadata") {
                            strong : format!("Created At: {}",
                                      report.metadata.creation_time);
                        }

                        div(class="row metadata") {
                            strong : format!("Last modified at: {}",
                                             report.metadata.modified_time);
                        }

                        div(class="row metadata") {
                            strong : format!("File Label: {}",
                                      &report.metadata.file_label);
                        }

                        div(class="row metadata") {
                            strong : format!("File Format Version: {}",
                                      report.metadata.file_format_version);
                        }

                        div(class="row metadata") {
                            @ if let Some(ref file_encoding) = &report.metadata.file_encoding {
                                strong : format!("File Encoding: {}", file_encoding);
                            }
                        }

                        div(class="row metadata") {
                            strong : format!("Compression type: {}",
                                      &report.metadata.compression);
                        }

                        br;

                        div(id="report", class="row") {
                            table(class="table table-bordered") {
                                tr {
                                    th(scope="col") : "Name";
                                    th(scope="col") : "Status";
                                    th(scope="col") : "Description";
                                }

                                @ for (name, status) in report.summary.iter() {
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

                        br;

                        div(class="row") {
                            h2(id="selected-check", class="d-none") : "hidden";
                        }

                        @ for (name, status) in report.summary.iter() {
                            @ if let Some(ref locators) = status.locator {
                                : locators_table(format!("{}", name),
                                                 locators.clone());
                            }
                        }
                    }

                    script(src="https://code.jquery.com/jquery-3.3.1.slim.min.js",
                           integrity="sha384-q8i/X+965DzO0rT7abK41JStQIAqVgRVzpbzo5smXKp4YfRvH+8abtTE1Pi6jizo",
                           crossorigin="anonymous") {}
                    script(src="https://cdnjs.cloudflare.com/ajax/libs/popper.js/1.14.3/umd/popper.min.js",
                           integrity="sha384-ZMP7rVo3mIykV+2+9J3UJ46jBk0WLaUAdn689aCwoqbBJiSnjAK/l8WvCWPIPm49",
                           crossorigin="anonymous") {}
                    script(src="https://stackpath.bootstrapcdn.com/bootstrap/4.1.1/js/bootstrap.min.js",
                           integrity="sha384-smHYKdLADwkXOn1EmN1qk/HfnUcbVRZyYmZ4qpPea6sjB/pTJ0euyQp0Mk8ck+5T",
                           crossorigin="anonymous") {}
                    script(type="text/javascript") {
                         : javascript;
                    }
                }
            }
        }
    )
}

fn locators_table<'a>(name: String, locators: HashSet<Locator>) -> Box<RenderBox> {
    box_html! {
        div(class="row") {
            table(id=name.to_lowercase().replace(" ", "_"),
                  class="table table-striped table-bordered d-none") {
                tr {
                    th(scope="col") : "# (limited to 1000)";
                    th(scope="col") : "Variable (Index)";
                    th(scope="col") : "Row Index";
                }

                @ for (i, pair) in locators.iter().take(1000).enumerate() {
                    tr(class="locator") {
                        td(scope="row") : i + 1;
                        td : format!("{} ({})",
                                     pair.variable_name,
                                     pair.variable_index);

                        : value_if_positive(pair.value_index, "-");
                    }
                }
            }
        }
    }
}

fn value_if_positive(value: i32, default: &'static str) -> Box<Render> {
    box_html! {
        @ if value < 0 {
            td : default;
        } else {
            td : value;
        }
    }
}

fn logo() -> Box<RenderBox + 'static> {
    box_html! {
        div(id="logo", class="row") {
            svg(xmlns="http://www.w3.org/2000/svg" ,
                    xmlns:xlink="http://www.w3.org/1999/xlink",
                    xmlns:krita="http://krita.org/namespaces/svg/krita",
                    xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd",
                    width="180pt",
                    height="42pt",
                    viewBox="-20 32 180 47") {

                    g(id="group0",
                      transform="translate(-27.26796875, 33.71999999)",
                      fill="none") {
                        text(id="shape0",
                             transform="translate(41.7640625,28.50000001)",
                             fill="#000000",
                             font-family="DejaVu Sans",
                             font-size="16",
                             font-size-adjust="0.473684") {
                            tspan(x="0") : "QAMyData"
                        }

                        g(id="group1",
                          transform="matrix(1 0 0 1.1666666672 0 0)",
                          fill="none") {
                            path(id="shape0",
                                 transform="matrix(0.52986604692 0 0 0.46108506123 0 0)",
                                 fill="#90d345",
                                 fill-rule="evenodd",
                                 stroke="#000000",
                                 stroke-opacity="0",
                                 stroke-width="4.5",
                                 stroke-linecap="square",
                                 stroke-linejoin="bevel",
                                 d="M34.1585 0L0 21.7714L0 58.5575L34.5339 78.0767L67.9417 58.1821L67.5663 18.7684Z");
                            path(id="shape1",
                                 transform="matrix(0.52986604692 0 0 0.46108506123 7.6943435757 11.769224091)",
                                 fill="none",
                                 stroke="#ffffff",
                                 stroke-width="9.75",
                                 stroke-linecap="square",
                                 stroke-linejoin="bevel",
                                 d="M0 11.4243L13.8789 26.2758L40.2825 0");
                        }

                        path(id="shape1",
                             transform="translate(39.59995, 1.2000000101)",
                             fill="#a6a6a6",
                             fill-rule="evenodd",
                             stroke="#999999",
                             stroke-width="1.2",
                             stroke-linecap="square",
                             stroke-linejoin="miter",
                             stroke-miterlimit="2",
                             d="M0.0001 39.6L0 0");
                    }

                }
        }
    }
}
