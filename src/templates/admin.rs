use axum_extra::extract::CookieJar;
use maud::{html, Markup, DOCTYPE};
use reqwest::StatusCode;

use crate::{
    config::CONFIG,
    database,
    templates::{render_monitor_list, NEWCSS},
};

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
        html {
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
        }
    );

    (StatusCode::OK, render)
}
