use axum_extra::extract::CookieJar;
use maud::{Markup, html};

use crate::{
    config::CONFIG,
    database::{self, record::RecordResult},
};
use maud::DOCTYPE;

async fn render_monitor_list() -> Markup {
    let mons = database::monitor::get_all().await.unwrap();
    html!(
        table {
            caption { "Enabled monitors" }
            thead { tr {
                th scope="col" { "ID" };
                th scope="col" { "Service" };
                th scope="col" { "Last checked" };
                th scope="col" { "Interval" };
                th scope="col" { "Enabled" };
            } }
            tbody {
                @for (id, mon) in mons {
                    tr {
                        @let last_record = crate::database::record::util_last_record(id).await.unwrap();
                        td { (id) };
                        td { (mon.service_data.service_location_str()) };
                        td {
                            @let (msg, style) = match last_record.result {
                                RecordResult::Ok => ("Up", "color: #6fff31"),
                                RecordResult::Unexpected => ("UX", "color: #f48421"),
                                RecordResult::Down => ("Down", "color: #cb0b0b"),
                                RecordResult::Err => ("Err", "color #550505"),
                            };
                            (crate::time_util::time_rel(last_record.time_checked as i64))
                            " ("
                            span style=(style) { (msg) }
                            ")"
                        };
                        td { (mon.interval_mins) " min" };
                        td { (mon.enabled) };
                    }
                }
            }
        }
    )
}

pub async fn index_template(cookies: CookieJar) -> Markup {
    let is_logged_in = match cookies.get("token") {
        None => false,
        Some(c) => database::session::is_valid(c.value())
            .await
            .unwrap_or(false),
    };

    if CONFIG.get().unwrap().lock().await.allow_guest {
        html!(
            (DOCTYPE);
            head {
                link rel="stylesheet" href="https://fonts.xz.style/serve/inter.css";
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@exampledev/new.css@1.1.2/new.min.css";
                script src="/index.js" {};
                title { (CONFIG.get().unwrap().lock().await.instance_name) };
            }

            body {
                header {
                    h1 { (CONFIG.get().unwrap().lock().await.instance_name) };
                    div style="position: absolute; top: 5px; right: 5px" {
                        @if !is_logged_in {
                            label for="password" { "Login: " }
                            input #password placeholder="password ...";
                            button style="background: #181818" onclick="onLogin()" { "OK" }
                        } @else {
                            p { "You are logged in" }
                        }
                    };
                }
                p {
                    (render_monitor_list().await)
                }
            }
        )
    } else {
        html!(
            (DOCTYPE);
            head {
                link rel="stylesheet" href="https://fonts.xz.style/serve/inter.css";
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@exampledev/new.css@1.1.2/new.min.css";
                script src="/index.js" {};
                title { (CONFIG.get().unwrap().lock().await.instance_name) };
            }

            body {
                header {
                    h1 { (CONFIG.get().unwrap().lock().await.instance_name) };
                    div style="position: absolute; top: 5px; right: 5px" {
                        @if !is_logged_in {
                            label for="password" { "Login: " }
                            input #password placeholder="password ...";
                            button style="background: #181818" onclick="onLogin()" { "OK" }
                        } @else {
                            p { "You are logged in" }
                        }
                    };
                }
                @if is_logged_in { p { "Log in to see this" } }
                @else { a href="/admin" { "Go to admin page" }}
            }
        )
    }
}
