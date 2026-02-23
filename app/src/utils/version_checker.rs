use minreq;
use std::env;

pub fn get_version() -> String {
    let owner = "SmthFail";
    let repo = "tiny_system_monitor";

    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );

    let current_version = env!("CARGO_PKG_VERSION").to_string();

    let response = minreq::get(&url)
        .with_header("User-Agent", "rust-minimal-version-checker")
        .send();

    let version = match response {
        Ok(resp) => {
            if resp.status_code == 200 {
                let body = resp.as_str().unwrap_or("");
                match extract_tag_name(body) {
                    Some(remote_version) if remote_version != current_version => {
                        format!("{} (remote {})", current_version, remote_version)
                    }
                    _ => format!("{} (latest)", current_version),
                }
            } else {
                eprintln!("Error: HTTP {}", resp.status_code);
                current_version
            }
        }
        Err(err) => {
            eprintln!("Network error: {}", err);
            current_version
        }
    };

    version
}

fn extract_tag_name(body: &str) -> Option<String> {
    for line in body.lines() {
        if line.trim_start().starts_with("\"tag_name\"") {
            let colon = line.find(':')?;
            let value_start = line[colon + 1..].find('"')? + colon + 2;
            let value_end = line[value_start..].find('"')? + value_start;
            return Some(line[value_start..value_end].trim_start_matches('v').to_string());
        }
    }
    None
}
