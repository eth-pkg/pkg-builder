use log::info;
use pkg_builder_init::http_client;

/// Parse a GitHub "owner/repo" from a source URL.
///
/// Handles URLs like:
/// - `https://github.com/owner/repo/archive/refs/tags/v1.0.0.tar.gz`
/// - `https://github.com/owner/repo.git`
/// - `https://github.com/owner/repo/releases/download/v1.0.0/file.tar.gz`
pub fn parse_github_repo(url: &str) -> Option<String> {
    let url = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))?;

    let parts: Vec<&str> = url.splitn(4, '/').collect();
    if parts.len() < 2 {
        return None;
    }
    let owner = parts[0];
    let repo = parts[1].strip_suffix(".git").unwrap_or(parts[1]);
    Some(format!("{}/{}", owner, repo))
}

/// Fetch the latest release tag from a GitHub repository.
///
/// Returns the tag name (e.g., "v1.15.0") and the version with "v" prefix stripped.
pub fn fetch_latest_release(
    owner_repo: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        owner_repo
    );
    let client = http_client();

    let mut request = client
        .get(&url)
        .header("Accept", "application/vnd.github+json");

    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let resp = request.send()?;
    if !resp.status().is_success() {
        return Err(format!(
            "GitHub API error (HTTP {}): {}",
            resp.status(),
            resp.text().unwrap_or_default()
        )
        .into());
    }

    let json: serde_json::Value = resp.json()?;
    let tag_name = json["tag_name"]
        .as_str()
        .ok_or("No tag_name in release response")?;

    let version = tag_name.strip_prefix('v').unwrap_or(tag_name);
    info!(
        "Latest release for {}: {} (version {})",
        owner_repo, tag_name, version
    );

    Ok((tag_name.to_string(), version.to_string()))
}

/// Fetch the commit SHA for a given tag.
pub fn fetch_tag_commit(owner_repo: &str, tag: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.github.com/repos/{}/git/ref/tags/{}",
        owner_repo, tag
    );
    let client = http_client();

    let mut request = client
        .get(&url)
        .header("Accept", "application/vnd.github+json");
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let resp = request.send()?;
    if !resp.status().is_success() {
        return Err(format!(
            "GitHub API error fetching tag {}: HTTP {}",
            tag,
            resp.status()
        )
        .into());
    }

    let json: serde_json::Value = resp.json()?;
    let obj_type = json["object"]["type"].as_str().unwrap_or("");
    let sha = json["object"]["sha"]
        .as_str()
        .ok_or("No sha in tag ref response")?;

    // If it's an annotated tag, we need to dereference to get the commit
    if obj_type == "tag" {
        let tag_url = format!(
            "https://api.github.com/repos/{}/git/tags/{}",
            owner_repo, sha
        );
        let mut request = client
            .get(&tag_url)
            .header("Accept", "application/vnd.github+json");
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        let resp = request.send()?;
        let json: serde_json::Value = resp.json()?;
        let commit_sha = json["object"]["sha"]
            .as_str()
            .ok_or("No sha in annotated tag response")?;
        Ok(commit_sha.to_string())
    } else {
        Ok(sha.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_repo_archive_url() {
        let url = "https://github.com/ethereum/go-ethereum/archive/refs/tags/v1.14.0.tar.gz";
        assert_eq!(
            parse_github_repo(url),
            Some("ethereum/go-ethereum".to_string())
        );
    }

    #[test]
    fn test_parse_github_repo_release_url() {
        let url = "https://github.com/sigp/lighthouse/releases/download/v5.0.0/lighthouse-v5.0.0-x86_64-unknown-linux-gnu.tar.gz";
        assert_eq!(parse_github_repo(url), Some("sigp/lighthouse".to_string()));
    }

    #[test]
    fn test_parse_github_repo_git_url() {
        let url = "https://github.com/prysmaticlabs/prysm.git";
        assert_eq!(
            parse_github_repo(url),
            Some("prysmaticlabs/prysm".to_string())
        );
    }

    #[test]
    fn test_parse_github_repo_non_github() {
        let url = "https://example.com/foo/bar.tar.gz";
        assert_eq!(parse_github_repo(url), None);
    }

    #[test]
    fn test_parse_github_repo_short() {
        let url = "https://github.com/owner/repo";
        assert_eq!(parse_github_repo(url), Some("owner/repo".to_string()));
    }
}
