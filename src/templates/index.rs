use axum_extra::extract::CookieJar;
use maud::{html, Markup, DOCTYPE};
use reqwest::StatusCode;

use crate::{
    config::CONFIG,
    database,
    templates::{render_monitor_list, HTML_HEADER_GLOB},
};

pub async fn index_template(cookies: CookieJar) -> (StatusCode, Markup) {
    let is_logged_in = match cookies.get("token") {
        None => false,
        Some(c) => database::session::is_valid(c.value())
            .await
            .unwrap_or_default(),
    };

    let allow_guest = CONFIG.get().unwrap().lock().await.allow_guest;
    let can_view = !(!allow_guest && !is_logged_in);

    let render = html! {
        (DOCTYPE);
        html {
            head {
                (HTML_HEADER_GLOB)
                script src="/static/index.js" {};
                title { (CONFIG.get().unwrap().lock().await.instance_name) }
            }

            body {
                header style="display: flex; align-items: center;" {
                    a href="/" {
                        img.logo src="/static/logo.png" alt="Logo";
                    }

                    h1 style="margin-bottom: 16px; margin-left: 16px; padding: 16px" { (CONFIG.get().unwrap().lock().await.instance_name) }

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
        }
    };

    let status = if can_view {
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    };

    (status, render)
}
