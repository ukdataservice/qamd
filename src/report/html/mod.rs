use horrorshow::helper::doctype;
use horrorshow::prelude::*;

use horrorshow::{Render, RenderBox};

use report::{Category, Metadata, Report, Status};

static JQUERY: &'static str = include_str!("../../../node_modules/jquery/jquery.min.js");
static BOOTSTRAP_CSS: &'static str =
    include_str!("../../../node_modules/bootstrap/dist/css/bootstrap.min.css");
static ANIMATE_CSS: &'static str =
    include_str!("../../../node_modules/animate.css/animate.min.css");
static JAVASCRIPT: &'static str = include_str!("custom.js");

pub trait IntoHtml {
    fn to_html(&self) -> String;
}

impl IntoHtml for Report {
    fn to_html(&self) -> String {
        format!(
            "{}",
            html! {
                : doctype::HTML;
                html {
                    head {
                        title : &self.metadata.file_name;
                        meta(charset="UTF-8");
                        style(type="text/css") {
                            : Raw(BOOTSTRAP_CSS);
                        }
                        style(type="text/css") {
                            : Raw(ANIMATE_CSS);
                        }
                    }

                    body {
                        div(class="container") {
                            : logo();

                            : metadata(&self.metadata);

                            br;

                            @ for category in Category::variants() {
                                div(id=format!("report-{:?}", category), class="row") {
                                    h2 : format!("{}", category);

                                    table(class="table table-bordered") {
                                        tr {
                                            th(scope="col") : "Name";
                                            th(scope="col") : "Status (N)";
                                            th(scope="col") : "Description";
                                        }

                                        @ for (name, status) in self.into_iter()
                                            .filter(|(_, status)| status.category == *category) {

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
                            }

                            div(class="row") {
                                h2(id="selected-check", class="d-none") : "hidden";
                            }

                            @ for (name, status) in self.into_iter() {
                                @ if status.locators.is_some() {
                                    : locators_table(format!("{}", name),
                                                     status.clone());
                                }
                            }
                        }

                        script(type="text/javascript") {
                            : Raw(JQUERY);
                        }
                        script(type="text/javascript") {
                             : Raw(JAVASCRIPT);
                        }
                    }
                }
            }
        )
    }
}

fn locators_table<'a>(name: String, status: Status) -> Box<RenderBox> {
    box_html! {
        div(class="row") {
            table(id=name.to_lowercase().replace(" ", "_"),
                  class="table table-striped table-bordered d-none") {
                tr {
                    th(scope="col") : "# (limited to 1000)";
                    th(scope="col") : "Variable";
                    th(scope="col") : "Row number";
                }

                @ for (i, pair) in status.into_iter().take(1000).enumerate() {
                    tr(class="locator") {
                        td(scope="row") : i + 1;
                        td : format!("{}", pair.variable_name);

                        : value_if_positive(pair.value_index, "-");
                    }
                }
            }
        }
    }
}

fn value_if_positive(value: i32, default: &'static str) -> Box<Render> {
    box_html! {
        @ if value <= 0 {
            td : default;
        } else {
            td : value;
        }
    }
}

// fn value_if_present(value: &Option<String>, default: String) -> String {
//     match value {
//         Some(v) => return v.to_string(),
//         None => return default,
//     }
// }

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
                             font-family="sans-serif",
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

fn metadata<'a>(metadata: &'a Metadata) -> Box<RenderBox + 'a> {
    box_html! {
        div(id="title", class="row") {
            h1(id="file-name") : &metadata.file_name;
        }

        div(class="row metadata") {
            strong : format!("Raw Case Count: {}",
                      metadata.raw_case_count);
        }

        div(class="row metadata") {
            @ if let Some(case_count) = metadata.case_count {
                strong : format!("Aggregated Case Count: {}",
                          case_count);
            }
        }

        div(class="row metadata") {
            strong : format!("Total Variables: {}",
                      metadata.variable_count);
        }

        div(class="row metadata") {
            strong : format!("Data Type Occurences: {:#?}",
                             metadata.data_type_occurences);
        }

        div(class="row metadata") {
            strong : format!("Created At: {}",
                      metadata.creation_time);
        }

        div(class="row metadata") {
            strong : format!("Last modified at: {}",
                             metadata.modified_time);
        }

        div(class="row metadata") {
            strong : format!("File Label: {}",
                      &metadata.file_label);
        }

        div(class="row metadata") {
            strong : format!("File Format Version: {}",
                      metadata.file_format_version);
        }

        div(class="row metadata") {
            @ if let Some(ref file_encoding) = &metadata.file_encoding {
                strong : format!("File Encoding: {}", file_encoding);
            }
        }

        div(class="row metadata") {
            strong : format!("Compression type: {}",
                      &metadata.compression);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use report::Metadata;

    #[test]
    fn test_metadata() {
        let mut mdata = Metadata::new();
        mdata.file_name = "test".to_string();

        let rendered = html! {
            :  metadata(&mdata);
        };

        let actual = r#"<div id="title" class="row"><h1 id="file-name">test</h1></div><div class="row metadata"><strong>Raw Case Count: 0</strong></div><div class="row metadata"></div><div class="row metadata"><strong>Total Variables: 0</strong></div><div class="row metadata"><strong>Data Type Occurences: {}</strong></div><div class="row metadata"><strong>Created At: 0</strong></div><div class="row metadata"><strong>Last modified at: 0</strong></div><div class="row metadata"><strong>File Label: </strong></div><div class="row metadata"><strong>File Format Version: 0</strong></div><div class="row metadata"></div><div class="row metadata"><strong>Compression type: </strong></div>"#;

        assert_eq!(format!("{}", rendered), actual.to_string());
    }
}
