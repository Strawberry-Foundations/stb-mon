use axum_extra::extract::CookieJar;
use maud::{Markup, html};

use crate::{config::CONFIG, database};
use maud::DOCTYPE;

pub async fn index(cookies: CookieJar) -> Markup {
    let is_logged_in = match cookies.get("token") {
        None => false,
        Some(c) => database::session::is_valid_session(c.value()).await.unwrap_or(false),
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
                p { "WIP" }
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
