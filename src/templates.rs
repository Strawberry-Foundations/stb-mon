use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use maud::{Markup, PreEscaped, html};

use crate::{
    config::CONFIG,
    database::{self, record::RecordResult},
};
use maud::DOCTYPE;

static NEWCSS: PreEscaped<&'static str> = PreEscaped(
    r#"
<link rel="stylesheet" href="https://fonts.xz.style/serve/inter.css">
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@exampledev/new.css@1.1.2/new.min.css">
<style>body { max-width: 65% }</style>
"#,
);

async fn render_monitor_list(admin: bool) -> Markup {
    let mons = database::monitor::get_all(false).await.unwrap();
    html!(
        table {
            caption { "Enabled monitors" }
            thead { tr {
                th scope="col" { "ID" };
                th scope="col" { "Service" };
                th scope="col" { "Last checked" };
                th scope="col" { "Interval" };
                th scope="col" { "Enabled" };
                @if admin { th scope="col" { "Actions" } };
            } }
            tbody {
                @for (id, mon) in mons {
                    tr {
                        @let last_record = crate::database::record::util_last_record(id).await.unwrap();
                        td { (id) };
                        td { (mon.service_data.service_location_str()) };
                        td {
                            @let (msg, color) = match last_record.result {
                                RecordResult::Ok => ("Up", "#6fff31"),
                                RecordResult::Unexpected => ("UX", "#f48421"),
                                RecordResult::Down => ("Down", "#cb0b0b"),
                                RecordResult::Err => ("Err", "#550505"),
                            };
                            (crate::time_util::time_rel(last_record.time_checked as i64))
                            " ("
                            span style={ "color: " (color) } {
                                (msg)
                            }

                            @if let Some(time) = last_record.response_time_ms {
                                " " (time) "ms"
                            }
                            ")";
                        };
                        td { (mon.interval_mins) " min" };
                        td { (mon.enabled) };
                        @if admin { td {
                            a href={ "javascript:onDelete(" (id) ")" } { "Del" };
                            " "
                            a href={ "javascript:onToggle(" (id) ")" } {
                                @if mon.enabled { "Dis" }
                                @else { "En" }
                            }
                        }}
                        td { a href={ "/monitor/" (id) } { "More" } }
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

    let pw_input = html!(
        div style="position: absolute; top: 5px; right: 5px" {
            @if !is_logged_in {
                label for="password" { "Login: " };
                input #password placeholder="password ..." type="password";
                button style="background: #181818" onclick="onLogin()" { "OK" };
            } @else {
                p { "You are logged in - " a href="/admin" { "ADMIN" } };
            }
        };
    );

    if CONFIG.get().unwrap().lock().await.allow_guest {
        html! {
            (DOCTYPE);
            head {
                (NEWCSS);
                script src="/index.js" {};
                title { (CONFIG.get().unwrap().lock().await.instance_name) };
            }

            body {
                header {
                    h1 { (CONFIG.get().unwrap().lock().await.instance_name) };
                    (pw_input)
                }
                p {
                    (render_monitor_list(false).await)
                }
            }
        }
    } else {
        html!(
            (DOCTYPE);
            head {
                script src="/index.js" {};
                title { (CONFIG.get().unwrap().lock().await.instance_name) };
            }

            body {
                header {
                    h1 { (CONFIG.get().unwrap().lock().await.instance_name) };
                    (pw_input)
                }
                @if is_logged_in { p { "Log in to see this" } }
                @else { a href="/admin" { "Go to admin page" } }
            }
        )
    }
}

pub async fn admin_template(cookies: CookieJar) -> (StatusCode, Markup) {
    let is_logged_in = match cookies.get("token") {
        None => false,
        Some(c) => database::session::is_valid(c.value())
            .await
            .unwrap_or(false),
    };
    if !is_logged_in {
        let render = html!(
            (DOCTYPE);
            head {
                (NEWCSS)
                title { "Unauthorized" }
            }

            body {
                header { "Unauthorized" }
                p { "Please log in to see this page" };
                a href="/" { "Back to main page" };
            }
        );

        return (StatusCode::UNAUTHORIZED, render);
    }

    let render = html!(
        (DOCTYPE);
        head {
            (NEWCSS);
            script src="/admin.js" {};
            title { (CONFIG.get().unwrap().lock().await.instance_name) };
        }

        body {
            header {
                h1 { (CONFIG.get().unwrap().lock().await.instance_name) " - Admin" };
            }
            p {
                (render_monitor_list(true).await)
            }
        }
    );

    return (StatusCode::OK, render);
}
