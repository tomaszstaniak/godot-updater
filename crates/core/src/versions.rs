use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Channel {
    Stable,
    Dev,
    Beta,
    RC,
    LTS,
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Channel::Stable => write!(f, "Stable"),
            Channel::Dev => write!(f, "Dev"),
            Channel::Beta => write!(f, "Beta"),
            Channel::RC => write!(f, "RC"),
            Channel::LTS => write!(f, "LTS"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Edition {
    Standard,
    Mono,
}

impl fmt::Display for Edition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Edition::Standard => write!(f, "Standard"),
            Edition::Mono => write!(f, "Mono"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GodotVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub channel: Channel,
    pub edition: Edition,
    pub tag: String,
    pub pre_release_num: Option<u32>,
}

impl GodotVersion {
    /// Parse a GitHub release tag into a GodotVersion.
    /// Examples: "4.6.1-stable", "4.7-dev3", "4.7-beta1", "4.7-rc1", "3.6-stable"
    pub fn parse(tag: &str, edition: Edition) -> Option<Self> {
        let tag = tag.strip_prefix('v').unwrap_or(tag);

        // Split on first '-' to get version and suffix
        let (version_part, suffix) = match tag.find('-') {
            Some(idx) => (&tag[..idx], &tag[idx + 1..]),
            None => return None,
        };

        let parts: Vec<&str> = version_part.split('.').collect();
        let major: u32 = parts.first()?.parse().ok()?;
        let minor: u32 = parts.get(1)?.parse().ok()?;
        let patch: u32 = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);

        let (channel, pre_release_num) = parse_suffix(suffix, major);

        Some(GodotVersion {
            major,
            minor,
            patch,
            channel,
            edition,
            tag: tag.to_string(),
            pre_release_num,
        })
    }

    /// Unique identifier for this version (without edition).
    pub fn version_string(&self) -> String {
        if self.patch == 0 {
            format!("{}.{}", self.major, self.minor)
        } else {
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        }
    }

    /// Sort key for ordering versions (higher = newer).
    fn sort_key(&self) -> (u32, u32, u32, u8, u32) {
        let channel_ord = match self.channel {
            Channel::Dev => 0,
            Channel::Beta => 1,
            Channel::RC => 2,
            Channel::Stable | Channel::LTS => 3,
        };
        (
            self.major,
            self.minor,
            self.patch,
            channel_ord,
            self.pre_release_num.unwrap_or(0),
        )
    }
}

impl Ord for GodotVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}

impl PartialOrd for GodotVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for GodotVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag)
    }
}

fn parse_suffix(suffix: &str, major: u32) -> (Channel, Option<u32>) {
    let suffix_lower = suffix.to_lowercase();

    if suffix_lower == "stable" {
        // 3.x is LTS
        let channel = if major <= 3 {
            Channel::LTS
        } else {
            Channel::Stable
        };
        return (channel, None);
    }

    if let Some(rest) = suffix_lower.strip_prefix("dev") {
        let num = rest.parse::<u32>().ok();
        return (Channel::Dev, num);
    }
    if let Some(rest) = suffix_lower.strip_prefix("beta") {
        let num = rest.parse::<u32>().ok();
        return (Channel::Beta, num);
    }
    if let Some(rest) = suffix_lower.strip_prefix("rc") {
        let num = rest.parse::<u32>().ok();
        return (Channel::RC, num);
    }

    // Fallback: treat as dev
    (Channel::Dev, None)
}

/// Sort versions descending (newest first).
pub fn sort_versions_desc(versions: &mut [GodotVersion]) {
    versions.sort_by(|a, b| b.cmp(a));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stable() {
        let v = GodotVersion::parse("4.6.1-stable", Edition::Standard).unwrap();
        assert_eq!(v.major, 4);
        assert_eq!(v.minor, 6);
        assert_eq!(v.patch, 1);
        assert_eq!(v.channel, Channel::Stable);
    }

    #[test]
    fn parse_dev() {
        let v = GodotVersion::parse("4.7-dev3", Edition::Standard).unwrap();
        assert_eq!(v.major, 4);
        assert_eq!(v.minor, 7);
        assert_eq!(v.patch, 0);
        assert_eq!(v.channel, Channel::Dev);
        assert_eq!(v.pre_release_num, Some(3));
    }

    #[test]
    fn parse_lts() {
        let v = GodotVersion::parse("3.6-stable", Edition::Standard).unwrap();
        assert_eq!(v.channel, Channel::LTS);
    }

    #[test]
    fn ordering() {
        let a = GodotVersion::parse("4.6-stable", Edition::Standard).unwrap();
        let b = GodotVersion::parse("4.6.1-stable", Edition::Standard).unwrap();
        assert!(b > a);
    }
}
