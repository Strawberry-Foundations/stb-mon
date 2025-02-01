use crate::{
    config::CONFIG,
    database::{self, record::RecordResult}, monitor::Monitor,
};

use axum::{extract::Path, http::StatusCode};
use axum_extra::extract::CookieJar;
use itertools::Itertools;
use maud::{html, Markup, PreEscaped, DOCTYPE};

#[rustfmt::skip]
static NEWCSS: PreEscaped<&'static str> = PreEscaped(concat!(
r#"<link rel="stylesheet" href="https://fonts.xz.style/serve/inter.css">"#,
r#"<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@exampledev/new.css@1.1.2/new.min.css">"#,
"<style>",
    "body {",
        "max-width: 65%;",
    "}",

    "#addform {",
        "input {",
            "min-width: 30%;",
            "display: block;",
        "}",

        "label {",
            "margin-down: 3px;",
            "display: block;",
        "}",
    "}",
"</style>"
));

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

pub async fn index_template(cookies: CookieJar) -> (StatusCode, Markup) {
    let is_logged_in = match cookies.get("token") {
        None => false,
        Some(c) => database::session::is_valid(c.value())
            .await
            .unwrap_or_default(),
    };

    let allow_guest = CONFIG.get().unwrap().lock().await.allow_guest;
    let can_view = allow_guest || (!allow_guest && is_logged_in);
    
    let render = html! {
        (DOCTYPE);
        head {
            (NEWCSS)
            script src="/index.js" {};
            title { (CONFIG.get().unwrap().lock().await.instance_name) }
        }
        
        body {
            header {
                h1 { (CONFIG.get().unwrap().lock().await.instance_name) }
                div style="position: absolute; top: 5px; right: 5px" {
                    @if !is_logged_in {
                        label for="password" { "Login: " }
                        input #password placeholder="Password" type="password";
                        button style="background: #181818" onclick="onLogin()" { "OK" }
                    } @else {
                        p { "You are logged in - " a href="/admin" { "ADMIN" } }
                    }
                };
            
            }

            @if can_view {
                body {
                    (render_monitor_list(false).await)
                }
            }
            @else {
                p { "Log in to see this" }
            }
        }
    };

    let status = if can_view {
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    };

    (status, render)
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
                header { h1 { "Unauthorized" } }
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

                        label for="service-name" { "Service name" }
                        input #service-name placeholder="e.g. Website, Blog";

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

                            label for="headers" { "Request headers" }
                            textarea #headers {}

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

    (StatusCode::OK, render)
}

async fn render_monitor_info(mon: Monitor) -> Markup {
    html!(
        div {
            style scoped { "th { width: 0; white-space: nowrap }" }

            table {
                caption { "General" }
                tr {
                    th { "Service name" }
                    td { (mon.service_name) }
                }
                tr {
                    th { "Service location" }
                    td { (mon.service_data.service_location_str()) }
                }
                tr {
                    th { "Enabled" }
                    td { (mon.enabled) }
                }
                tr {
                    th { "Check interval" }
                    td { (mon.interval_mins) "min" }
                }
                tr {
                    th { "Timeout" }
                    td { (mon.timeout_secs) "s" }
                }
            }

            table {
                caption { "Monitor-specific" }
                @for (k, v) in mon.service_data.as_hashmap().into_iter().sorted() {
                    tr {
                        th { (k) }
                        td { (v) }
                    }
                }
            }
        }
    )
}

pub async fn monitor_template(monitor_id: Path<u64>, cookies: CookieJar) -> (StatusCode, Markup) {
    let is_logged_in = match cookies.get("token") {
        None => false,
        Some(c) => database::session::is_valid(c.value())
            .await
            .unwrap_or_default(),
    };

    let allow_guest = CONFIG.get().unwrap().lock().await.allow_guest;
    let can_view = allow_guest || (!allow_guest && is_logged_in);

    let Some(monitor) = database::monitor::get_by_id(*monitor_id).await else {
        let render = html!(
            head {
                (NEWCSS)
            }

            body {
                header { h1 { "404 Not Found" } }
                p { "A monitor with this ID was not found" }
            }
        );

        return (
            StatusCode::NOT_FOUND,
            render,
        );
    };

    let render = html!(
        head {
            (NEWCSS)
            @if can_view { title { "Monitor " (monitor_id.to_string()) } }
            @else { title { "Unauthorized" } }
        }

        @if can_view {
            header {
                @let mon_name = if monitor.service_name.is_empty() {
                    &monitor.service_data.service_location_str()
                } else {
                    &monitor.service_name
                };
                @let mon_name = mon_name.split_at_checked(24).map(|s| s.0).unwrap_or(&mon_name);

                h1 { "Monitor info: " (mon_name) }
            }
            (render_monitor_info(monitor).await)
        }
        @else {
            header { h1 { "Unauthorized" } }
        }
    );

    let status = if can_view {
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    };

    (status, render)
}