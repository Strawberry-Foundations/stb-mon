use crate::{
    config::CONFIG,
    database::{
        self,
        record::{MonitorRecord, RecordResult},
    },
    monitor::Monitor,
    templates::{result_to_text_color, HTML_HEADER_GLOB},
    time_util::{self, current_unix_time},
};

use axum::extract::Path;
use axum_extra::extract::CookieJar;
use itertools::Itertools;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use reqwest::StatusCode;

#[allow(clippy::let_unit_value)]
async fn render_monitor_info(mon: Monitor, mon_id: u64) -> Markup {
    let time = current_unix_time();

    let Ok(records) = database::record::records_from_mon(mon_id).await else {
        return html!(p { (format!("Internal server error")) });
    };

    html!(
        div {
            style scoped { "th { width: 0; white-space: nowrap }" }

            table {
                caption { "General" }
                tr {
                    th scope="row" { "Service name" }
                    td { (mon.service_name) }
                }
                tr {
                    th scope="row" { "Service location" }
                    td { (mon.service_data.service_location_str()) }
                }
                tr {
                    th scope="row" { "Enabled" }
                    td { (mon.enabled) }
                }
                tr {
                    th scope="row" { "Check interval" }
                    td { (mon.interval_mins) "min" }
                }
                tr {
                    th scope="row" { "Timeout" }
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

            table {
                caption { "Uptime" }
                thead {
                    tr {
                        th scope="col" { "Time span" };
                        th scope="col" { "Status" }
                        th scope="col" { "Response time" }
                    }
                }

                @let first_record_time = records.last().unwrap().time_checked;
                tr {
                    th scope="row" { "Current" }
                    @let last_record = records.first().unwrap();
                    @let (msg, color) = result_to_text_color(&last_record.result);
                    @let last_same_status = records
                        .iter()
                        .position(|r| r.result != last_record.result)
                        .map(|i| records[i - 1].time_checked)
                        .unwrap_or(first_record_time);

                    td { span style={ "color:" (color) } { (msg) } " for " (time_util::time_diff_now(last_same_status as _)) }
                    td { (last_record.response_time_ms.map(|n| n.to_string()).unwrap_or_else(|| "N/A ".to_string())) "ms" }
                }
                
                @let records_last_30d = records
                    .iter()
                    .filter(|r| r.time_checked >= 60 * 60 * 24 * 30)
                    .collect::<Vec<&MonitorRecord>>();

                @for (timespan, t) in [
                    ("4h", 60 * 60 * 4),
                    ("12h", 60 * 60 * 12),
                    ("24h", 60 * 60 * 24),
                    ("72h", 60 * 60 * 72),
                    ("7d", 60 * 60 * 24 * 7),
                    ("14d", 60 * 60 * 24 * 14),
                    ("30d", 60 * 60 * 24 * 30)
                ].iter().filter(|(_, t)| *t < time_util::current_unix_time() - first_record_time) {
                    tr {
                        th scope="row" { "Last " (timespan) }
                        @let records = records_last_30d.iter().filter(|r| r.time_checked > time - t).collect::<Vec<&&MonitorRecord>>();

                        @let (amount_ok, amount_ux, amount_down, amount_err) = records.iter().fold((0f32, 0f32, 0f32, 0f32), |mut amounts, r| {
                            match r.result {
                                RecordResult::Ok => amounts.0 += 1.,
                                RecordResult::Unexpected => amounts.1 += 1.,
                                RecordResult::Down => amounts.2 += 1.,
                                RecordResult::Err => amounts.3 += 1.,
                            };

                            amounts
                        });

                        @let perc_ok = amount_ok / records.len() as f32 * 100.;
                        @let perc_ux = amount_ux / records.len() as f32 * 100.;
                        @let perc_down = amount_down / records.len() as f32 * 100.;
                        @let perc_err = amount_err / records.len() as f32 * 100.;

                        @let mut statuses: Vec<String> = vec![];
                        @for (s, p) in [
                            (RecordResult::Ok, perc_ok),
                            (RecordResult::Unexpected, perc_ux),
                            (RecordResult::Down, perc_down),
                            (RecordResult::Err, perc_err)
                        ] {
                            @if p > 0. {
                                @let (msg, color) = result_to_text_color(&s);
                                @let () = statuses.push(html!(span style={ "color:" (color) } { (format!("{p:.2}")) "% " (msg) }).into_string());
                            }
                        }

                        td { (PreEscaped(statuses.join(" "))) }

                        @let response_times = records.iter().filter_map(|r| r.response_time_ms).collect::<Vec<u64>>();
                        
                        @if response_times.is_empty() {
                            td { "N/A" }
                        } @else {
                            @let lowest_response_time = response_times.iter().min().unwrap();
                            @let highest_response_time = response_times.iter().max().unwrap();
                            @let avg_response_time = response_times.iter().sum::<u64>() / records.len() as u64;

                            td { "L: " (lowest_response_time) "ms H: " (highest_response_time) "ms Avg: " (avg_response_time) "ms" }
                        }
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
    let can_view = !(!allow_guest && !is_logged_in);

    let Some(monitor) = database::monitor::get_by_id(*monitor_id).await else {
        let render = html!(
            (DOCTYPE)
            html {
                head {
                    (HTML_HEADER_GLOB)
                    title { "Not found" }
                }

                body {
                    header { h1 { "404 Not Found" } }
                    p { "A monitor with this ID was not found" }
                }
            }
        );

        return (StatusCode::NOT_FOUND, render);
    };

    let render = html!(
        (DOCTYPE)
        html {
            head {
                (HTML_HEADER_GLOB)
                @if can_view { title { (monitor.service_name) " - " (CONFIG.get().unwrap().lock().await.instance_name) } }
                @else { title { "Unauthorized" } }
            }

            @if can_view {
                @let mon_name = if monitor.service_name.is_empty() {
                    &monitor.service_data.service_location_str()
                } else {
                    &monitor.service_name
                };
                @let mon_name = mon_name.split_at_checked(24).map_or(mon_name.as_str(), |s| s.0);

                header style="display: flex; align-items: center;" {
                    a href="/" {
                        img.logo src="/static/logo.png" alt="Logo";
                    }

                    h1 style="margin-bottom: 16px; margin-left: 16px; padding: 16px" { "Monitor info: " (mon_name) }
                }
                (render_monitor_info(monitor, *monitor_id).await)
            }
            @else {
                header { h1 { "Unauthorized" } }
            }
        }
    );

    let status = if can_view {
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    };

    (status, render)
}
