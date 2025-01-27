use crate::{
    config::CONFIG,
    database::{self, record::RecordResult},
};

use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use itertools::Itertools;
use maud::{DOCTYPE, Markup, PreEscaped, html};

static NEWCSS: PreEscaped<&'static str> = PreEscaped(
    r#"
<link rel="stylesheet" href="https://fonts.xz.style/serve/inter.css">
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@exampledev/new.css@1.1.2/new.min.css">
<style>
    body {
        max-width: 65%;
    }

    input, label {
        display: block;
    }

    input {
        min-width: 30%;
    }

    label {
        margin-down: 3px;
    }
</style>
"#
);

async fn render_monitor_list(admin: bool) -> Markup {
    let mons = database::monitor::get_all(false).await.unwrap();
    let mons = mons
        .into_iter()
        .sorted_by(|(i1, _), (i2, _)| i1.cmp(i2))
        .sorted_by(|(_, m1), (_, m2)| m2.enabled.cmp(&m1.enabled));
    html!(
        table {
            caption { "Monitors" }
            thead { tr {
                @if admin { th scope="col" { "ID" } }
                th scope="col" { "Service" }
                th scope="col" { "Last checked" }
                th scope="col" { "Interval" }
                th scope="col" { "Enabled" }
                @if admin { th scope="col" { "Actions" } }
            } }
            tbody {
                @for (id, mon) in mons {
                    tr {
                        @let last_record = crate::database::record::util_last_record(id).await.unwrap();
                        @if admin { td { (id) } }
                        td {
                            @let loc = mon.service_data.service_location_str();
                            @let tloc = loc.split_at_checked(126).map(|s| s.0).unwrap_or(&loc);
                            @if loc.starts_with("http") {
                                a href=(loc) { (tloc) }
                            }
                            @else {
                                (tloc)
                            }
                        }
                        td {
                            @let (msg, color) = match last_record.result {
                                RecordResult::Ok => ("Up", "#6fff31"),
                                RecordResult::Unexpected => ("UX", "#f48421"),
                                RecordResult::Down => ("Down", "#cb0b0b"),
                                RecordResult::Err => ("Err", "#550505"),
                            };
                            (crate::time_util::time_rel(last_record.time_checked as i64))
                            " ("
                            span title=(last_record.info) style={ "color: " (color) } {
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
            .unwrap_or_default(),
    };

    let pw_input = html!(
        div style="position: absolute; top: 5px; right: 5px" {
            @if !is_logged_in {
                label for="password" { "Login: " }
                input #password placeholder="Password" type="password";
                button style="background: #181818" onclick="onLogin()" { "OK" }
            } @else {
                p { "You are logged in - " a href="/admin" { "ADMIN" } }
            }
        };
    );

    if CONFIG.get().unwrap().lock().await.allow_guest {
        html! {
            (DOCTYPE);
            head {
                (NEWCSS)
                script src="/index.js" {};
                title { (CONFIG.get().unwrap().lock().await.instance_name) }
            }

            body {
                header {
                    h1 { (CONFIG.get().unwrap().lock().await.instance_name) }
                    (pw_input)
                }
                p {
                    (render_monitor_list(false).await)
                }
            }
        }
    } else {
        html!(
            (DOCTYPE)
            head {
                script src="/index.js" {};
                title { (CONFIG.get().unwrap().lock().await.instance_name) }
            }

            body {
                header {
                    h1 { (CONFIG.get().unwrap().lock().await.instance_name) }
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
            .unwrap_or_default(),
    };
    if !is_logged_in {
        let render = html!(
            (DOCTYPE)
            head {
                (NEWCSS)
                title { "Unauthorized" }
            }

            body {
                header { "Unauthorized" }
                p { "Please log in to see this page" }
                a href="/" { "Back to main page" }
            }
        );

        return (StatusCode::UNAUTHORIZED, render);
    }

    let render = html!(
        (DOCTYPE);
        head {
            (NEWCSS);
            script src="/admin.js" {};
            title { (CONFIG.get().unwrap().lock().await.instance_name) }
        }

        body {
            header {
                h1 { (CONFIG.get().unwrap().lock().await.instance_name) " - Admin" }
            }
            p {
                (render_monitor_list(true).await)
                details {
                    summary { "Add" };
                    form #addform autocomplete="off" action="javascript:onAdd()" {
                        label for="service-type" { "Type" }
                        select #service-type onchange="onAddTypeChange()" {
                            option value="tcp" selected { "TCP" }
                            option value="http" { "HTTP" }
                        }

                        label for="interval" { "Check interval" }
                        input #interval type="number" placeholder="minutes" min="1" max="94080" value="10";

                        label for="timeout" { "Timeout" }
                        input #timeout type="number" placeholder="seconds" min="1" max="60" value="5";
                        
                        br;

                        div #tcp-options {
                            label for="sock-addr" { "Socket address" }
                            input #sock-addr placeholder="133.33.33.37:4269";

                            label for="expected-response" { "Expected response" }
                            select #tcp-expected-response /* onchanged="onTcpExpectedResponseChange()" */ {
                                option value="op" { "Open port" }
                                option value="bytes" disabled { "Bytes" }
                            }

                            // TODO: bytes input fields
                        }

                        div #http-options hidden {
                            label for="method" { "Method" }
                            select #method {
                                option value="get" { "GET" }
                                option value="post" { "POST" }
                                option value="put" { "PUT" }
                                option value="delete" { "DELETE" }
                                option value="options" { "OPTIONS" }
                                option value="head" { "HEAD" }
                                option value="trace" { "TRACE" }
                                option value="connect" { "CONNECT" }
                                option value="patch" { "PATCH" }
                            }

                            label for="url" { "URL" }
                            input #url placeholder="https://example.com";

                            label for="expected-response" { "Expected response" }
                            select #http-expected-response onchange="onHttpExpectedResponseChange()" {
                                option value="any" { "Any" }
                                option value="sc" { "Status codes" }
                                option value="res" disabled { "Specific response" }
                            };

                            div #http-sc-options hidden {
                                label for="status-code" { "Status codes" }
                                input #status-code placeholder="200-299, 301, 400-410";
                            };

                            div #http-response-options hidden {
                                // TODO: front end for body checksum generation
                                label for="body-cs" { "Response body adler32 hash" }
                                input #body-cs type="number" placeholder="adler32" min="0" max="4294967296";
                            };

                            label for="request-body" { "Request body" }
                            textarea #request-body {};


                        };
                        br;

                        input type="submit";
                    }
                }
            }
        }
    );

    return (StatusCode::OK, render);
}
