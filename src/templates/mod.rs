use itertools::Itertools;
use maud::{html, Markup, PreEscaped};

use crate::{
    database::{self, record::RecordResult},
    time_util,
};

mod admin;
mod index;
mod monitor;

pub use admin::admin_template;
pub use index::index_template;
pub use monitor::monitor_template;

#[rustfmt::skip]
static HTML_HEADER_GLOB: PreEscaped<&'static str> = PreEscaped(concat!(
r#"<link rel="stylesheet" href="https://fonts.xz.style/serve/inter.css">"#,
r#"<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@exampledev/new.css@1.1.2/new.min.css">"#,
r#"<link rel="shortcut icon" type="image/png" href="/static/favicon.png">"#,
"<style>",
    "body { max-width: 65%; }",
    "#addform {",
        "input { min-width: 30%; display: block; }",
        "label { margin-down: 3px; display: block; }",
    "}",
    ".logo { image-rendering: pixelated; margin-bottom: 0; height: 48px }",
    "header { padding-top: 0.5rem; padding-bottom: 0.2rem; }",
"</style>"
));

fn result_to_text_color(res: &RecordResult) -> (&'static str, &'static str) {
    match res {
        RecordResult::Ok => ("Up", "#6fff31"),
        RecordResult::Unexpected => ("UX", "#f48421"),
        RecordResult::Down => ("Down", "#cb0b0b"),
        RecordResult::Err => ("Err", "#550505"),
    }
}

async fn render_monitor_list(admin: bool) -> Markup {
    let mons = database::monitor::get_all(false).await.unwrap();
    let mons = mons
        .into_iter()
        .sorted_by(|(i1, _), (i2, _)| i1.cmp(i2))
        .sorted_by(|(_, m1), (_, m2)| m2.enabled.cmp(&m1.enabled));

    html!(
        table {
            caption { "Monitors" }
            thead {
                tr {
                    @if admin { th scope="col" { "ID" } }
                    th scope="col" { "Service name" }
                    th scope="col" { "Service location" }
                    th scope="col" { "Last checked" }
                    th scope="col" { "Interval" }
                    @if admin {
                        th scope="col" { "Enabled" }
                        th scope="col" { "Actions" }
                    }
                }
            }
            tbody {
                @for (id, mon) in mons {
                    @let Ok(last_record) = crate::database::record::util_last_record(id).await else {
                        continue;
                    };

                    @let background_color = match last_record.result {
                        RecordResult::Ok => "rgba(0, 0, 0, 0)",
                        RecordResult::Unexpected => "rgba(245, 204, 0, 0.1)",
                        RecordResult::Down => "rgba(255, 0, 0, 0.1)",
                        RecordResult::Err => "rgba(125, 21, 21, 0.1)",
                    };

                    tr style={ "background-color:" (background_color) } {
                        @if admin { td { (id) } }
                        td { (mon.service_name) }
                        td {
                            @let loc = mon.service_data.service_location_str();
                            @let tloc = loc.split_at_checked(126).map_or(loc.as_str(), |s| s.0);
                            @if loc.starts_with("http") {
                                a href=(loc) { (tloc) }
                            }
                            @else {
                                (tloc)
                            }
                        }
                        td {
                            @let (msg, color) = result_to_text_color(&last_record.result);
                            (time_util::time_diff_now(last_record.time_checked as i64))
                            " ago ("
                            span title=(last_record.info) style={ "color: " (color) } {
                                (msg)
                            }

                            @if let Some(time) = last_record.response_time_ms {
                                " " (time) "ms"
                            }
                            ")";
                        };
                        td { (mon.interval_mins) " min" };
                        @if admin {
                            td { (mon.enabled) };
                            td {
                                a href={ "javascript:onDelete(" (id) ")" } { "Del" };
                                " "
                                a href={ "javascript:onToggle(" (id) ")" } {
                                    @if mon.enabled { "Dis" }
                                    @else { "En" }
                                }
                            }
                        }
                        td { a href={ "/monitor/" (id) } { "More" } }
                    }
                }
            }
        }
    )
}
