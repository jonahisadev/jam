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
    pub protocol: Vec<String>,

    /// Restrict to specific country
    #[arg(long, short)]
    pub country: Option<String>,

    /// Highest acceptable sync delay in seconds (defaults to 3600)
    #[arg(long, short)]
    pub delay: Option<u32>,

    /// Maximum mirrors to leave uncommented in mirrorlist
    #[arg(long, short = 'n')]
    pub maximum_mirrors: Option<usize>,
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

fn maybe_absent_compare<T: PartialEq>(argument: &Option<T>, given: &T) -> bool {
    if let Some(unwrapped) = argument {
        unwrapped == given
    } else {
        true
    }
}

fn maybe_absent_list<T: PartialEq>(argument: &Vec<T>, given: &T) -> bool {
    if argument.len() == 0 {
        return true;
    }

    return argument.contains(&given);
}

fn ip_filter(args: (bool, bool), given: (bool, bool)) -> bool {
    if args.0 && !given.0 {
        return false;
    }

    if args.1 && !given.1 {
        return false;
    }

    true
}

pub fn process_mirrors(res: ApiResponse, args: &Args) -> Vec<Url> {
    let mut mirrors: Vec<Url> = res
        .urls
        .into_iter()
        .filter(|m| {
            // These can be null in the API response. If they are, we don't want
            // to use them because we can't calculate anything valuable.
            if m.duration_stddev.is_none() || m.duration_stddev.is_none() || m.score.is_none() {
                return false;
            }

            maybe_absent_compare(&args.country, &m.country_code)
                && maybe_absent_list(&args.protocol, &m.protocol)
                && m.completion_pct == Some(1.0)
                && m.delay <= Some(args.delay.unwrap_or(3600))
                && m.duration_avg.unwrap() + m.duration_stddev.unwrap() <= 1.0
        })
        .filter(|m| ip_filter((args.require_ipv4, args.require_ipv6), (m.ipv4, m.ipv6)))
        .collect();
    mirrors.sort_by(|a, b| a.score.unwrap().partial_cmp(&b.score.unwrap()).unwrap());
    mirrors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_value_is_the_same() {
        assert_eq!(maybe_absent_compare(&Some(42), &42), true);
    }

    #[test]
    fn compare_value_is_not_the_same() {
        assert_eq!(maybe_absent_compare(&Some(42), &0), false);
    }

    #[test]
    fn compare_value_none() {
        assert_eq!(maybe_absent_compare(&None, &42), true);
    }

    #[test]
    fn compare_list_does_contain() {
        assert_eq!(maybe_absent_list(&vec!["https"], &"https"), true);
    }

    #[test]
    fn compare_list_does_not_contain() {
        assert_eq!(maybe_absent_list(&vec!["https"], &"rsync"), false);
    }

    #[test]
    fn compare_list_is_empty() {
        assert_eq!(maybe_absent_list(&vec![], &"https"), true);
    }

    #[test]
    fn require_ipv4() {
        let args: (bool, bool) = (true, false);

        let has_ipv4: (bool, bool) = (true, false);
        let no_ipv4: (bool, bool) = (false, true);
        let has_both: (bool, bool) = (true, true);
        let has_neither: (bool, bool) = (false, false);

        assert_eq!(ip_filter(args, has_ipv4), true);
        assert_eq!(ip_filter(args, no_ipv4), false);
        assert_eq!(ip_filter(args, has_both), true);
        assert_eq!(ip_filter(args, has_neither), false);
    }

    #[test]
    fn require_ipv6() {
        let args: (bool, bool) = (false, true);

        let has_ipv6: (bool, bool) = (false, true);
        let no_ipv6: (bool, bool) = (true, false);
        let has_both: (bool, bool) = (true, true);
        let has_neither: (bool, bool) = (false, false);

        assert_eq!(ip_filter(args, has_ipv6), true);
        assert_eq!(ip_filter(args, no_ipv6), false);
        assert_eq!(ip_filter(args, has_both), true);
        assert_eq!(ip_filter(args, has_neither), false);
    }

    #[test]
    fn require_both_ip() {
        let args: (bool, bool) = (true, true);

        let has_ipv4: (bool, bool) = (true, false);
        let no_ipv4: (bool, bool) = (false, true);
        let has_both: (bool, bool) = (true, true);
        let has_neither: (bool, bool) = (false, false);

        assert_eq!(ip_filter(args, has_ipv4), false);
        assert_eq!(ip_filter(args, no_ipv4), false);
        assert_eq!(ip_filter(args, has_both), true);
        assert_eq!(ip_filter(args, has_neither), false);
    }

    #[test]
    fn require_neither_ip() {
        let args: (bool, bool) = (false, false);

        let has_ipv4: (bool, bool) = (true, false);
        let no_ipv4: (bool, bool) = (false, true);
        let has_both: (bool, bool) = (true, true);
        let has_neither: (bool, bool) = (false, false);

        assert_eq!(ip_filter(args, has_ipv4), true);
        assert_eq!(ip_filter(args, no_ipv4), true);
        assert_eq!(ip_filter(args, has_both), true);
        assert_eq!(ip_filter(args, has_neither), true);
    }

    #[test]
    fn test_main() {
        let res: ApiResponse = ApiResponse {
            urls: vec![
                Url {
                    url: String::from("https://test.com/"),
                    protocol: String::from("https"),
                    country_code: String::from("US"),
                    ipv4: true,
                    ipv6: false,
                    delay: Some(600),
                    score: Some(1.23),
                    completion_pct: Some(1.0),
                    duration_avg: Some(0.3),
                    duration_stddev: Some(0.4),
                },
                Url {
                    url: String::from("https://invalid.com/"),
                    protocol: String::from("https"),
                    country_code: String::from("US"),
                    ipv4: true,
                    ipv6: false,
                    delay: Some(600),
                    score: Some(1.23),
                    completion_pct: Some(1.0),
                    duration_avg: Some(0.9),
                    duration_stddev: Some(0.4),
                },
                Url {
                    url: String::from("http://invalid-2.com/"),
                    protocol: String::from("http"),
                    country_code: String::from("US"),
                    ipv4: true,
                    ipv6: false,
                    delay: Some(600),
                    score: Some(1.23),
                    completion_pct: Some(1.0),
                    duration_avg: Some(0.1),
                    duration_stddev: Some(0.1),
                },
                Url {
                    url: String::from("https://invalid-3.com/"),
                    protocol: String::from("https"),
                    country_code: String::from("US"),
                    ipv4: true,
                    ipv6: false,
                    delay: Some(600),
                    score: Some(1.23),
                    completion_pct: Some(0.99),
                    duration_avg: Some(0.1),
                    duration_stddev: Some(0.1),
                },
                Url {
                    url: String::from("https://invalid-4.com/"),
                    protocol: String::from("https"),
                    country_code: String::from("US"),
                    ipv4: true,
                    ipv6: false,
                    delay: Some(6000),
                    score: Some(1.23),
                    completion_pct: Some(1.0),
                    duration_avg: Some(0.1),
                    duration_stddev: Some(0.1),
                },
                Url {
                    url: String::from("https://invalid-5.ca/"),
                    protocol: String::from("https"),
                    country_code: String::from("CA"),
                    ipv4: true,
                    ipv6: false,
                    delay: Some(600),
                    score: Some(1.23),
                    completion_pct: Some(1.0),
                    duration_avg: Some(0.1),
                    duration_stddev: Some(0.1),
                },
            ],
        };

        let args: Args = Args {
            protocol: vec![String::from("https")],
            require_ipv6: false,
            require_ipv4: true,
            delay: None,
            maximum_mirrors: None,
            output: None,
            country: Some(String::from("US")),
        };

        let mirrors = process_mirrors(res, &args);

        assert_eq!(mirrors.len(), 1);
        assert_eq!(mirrors[0].url, "https://test.com/");
    }
}
