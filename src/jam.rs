use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(version, about = "Generate a mirrorlist for Arch Linux packages")]
pub struct Args {
    /// Where to write the results
    #[arg(long, short)]
    pub output: Option<String>,

    /// Use IPv6 (defaults to true)
    #[arg(long, default_value_t = false)]
    pub require_ipv6: bool,

    /// Use IPv4 (defaults to true)
    #[arg(long, default_value_t = true)]
    pub require_ipv4: bool,

    /// Specify [http, https, rsync, ftp]
    #[arg(long, short)]
    pub protocol: Option<String>,

    /// Restrict to specific country
    #[arg(long, short)]
    pub country: Option<String>,

    /// Highest acceptable sync delay in seconds (defaults to 3600)
    #[arg(long, short)]
    pub delay: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Url {
    pub url: String,
    country_code: String,
    protocol: String,
    completion_pct: Option<f32>,
    delay: Option<u32>,
    ipv4: bool,
    ipv6: bool,
    duration_avg: Option<f32>,
    duration_stddev: Option<f32>,
    score: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    pub urls: Vec<Url>,
}

pub fn maybe_absent_compare(argument: &Option<String>, given: &String) -> bool {
    if let Some(unwrapped) = argument {
        unwrapped == given
    } else {
        true
    }
}

pub fn process_mirrors(res: ApiResponse, args: &Args) -> Vec<Url> {
    let mut mirrors: Vec<Url> = res
        .urls
        .into_iter()
        .filter(|m| {
            if m.duration_stddev.is_none() || m.duration_stddev.is_none() || m.score.is_none() {
                return false;
            }

            maybe_absent_compare(&args.country, &m.country_code)
                && maybe_absent_compare(&args.protocol, &m.protocol)
                && m.completion_pct == Some(1.0)
                && m.delay <= Some(args.delay.unwrap_or(3600))
                && m.duration_avg.unwrap() + m.duration_stddev.unwrap() <= 1.0
        })
        .filter(|m| {
            if args.require_ipv4 && !m.ipv4 {
                return false;
            }

            if args.require_ipv6 && !m.ipv6 {
                return false;
            }

            true
        })
        .collect();
    mirrors.sort_by(|a, b| a.score.unwrap().partial_cmp(&b.score.unwrap()).unwrap());
    mirrors
}
