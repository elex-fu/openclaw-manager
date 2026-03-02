//! Semantic version parsing and comparison
//!
//! Supports format: MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]
//! Examples: 1.0.0, 1.0.0-beta, 1.0.0+build123

use anyhow::{Context, Result};
use std::cmp::Ordering;

/// Semantic version
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
    pub build: Option<String>,
}

impl Version {
    /// Parse version string
    pub fn parse(version_str: &str) -> Result<Self> {
        // Remove 'v' or 'V' prefix
        let version_str = version_str.trim_start_matches(['v', 'V']);

        // Split build metadata
        let (version_part, build) = if let Some(pos) = version_str.find('+') {
            let (v, b) = version_str.split_at(pos);
            (v, Some(b[1..].to_string()))
        } else {
            (version_str, None)
        };

        // Split prerelease
        let (version_part, prerelease) = if let Some(pos) = version_part.find('-') {
            let (v, p) = version_part.split_at(pos);
            (v, Some(p[1..].to_string()))
        } else {
            (version_part, None)
        };

        // Parse MAJOR.MINOR.PATCH
        let parts: Vec<&str> = version_part.split('.').collect();

        if parts.len() < 2 {
            return Err(anyhow::anyhow!(
                "Invalid version format: {}. Expected MAJOR.MINOR[.PATCH]",
                version_str
            ));
        }

        let major = parts[0]
            .parse::<u32>()
            .with_context(|| format!("Invalid major version: {}", parts[0]))?;

        let minor = parts[1]
            .parse::<u32>()
            .with_context(|| format!("Invalid minor version: {}", parts[1]))?;

        let patch = if parts.len() >= 3 {
            parts[2]
                .parse::<u32>()
                .with_context(|| format!("Invalid patch version: {}", parts[2]))?
        } else {
            0
        };

        Ok(Self {
            major,
            minor,
            patch,
            prerelease,
            build,
        })
    }

    /// Check if prerelease
    pub fn is_prerelease(&self) -> bool {
        self.prerelease.is_some()
    }

    /// Get version string
    pub fn to_string(&self) -> String {
        let mut s = format!("{}.{}.{}", self.major, self.minor, self.patch);

        if let Some(ref pre) = self.prerelease {
            s.push('-');
            s.push_str(pre);
        }

        if let Some(ref build) = self.build {
            s.push('+');
            s.push_str(build);
        }

        s
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare MAJOR.MINOR.PATCH
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }

        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }

        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Handle prerelease comparison
        // Rule: version with prerelease < version without prerelease
        // prerelease compared alphabetically
        match (&self.prerelease, &other.prerelease) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater, // 1.0.0 > 1.0.0-beta
            (Some(_), None) => Ordering::Less,    // 1.0.0-beta < 1.0.0
            (Some(a), Some(b)) => a.cmp(b),       // Alphabetically
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.prerelease.is_none());

        let v = Version::parse("v1.2.3").unwrap();
        assert_eq!(v.major, 1);
    }

    #[test]
    fn test_version_comparison() {
        assert!(Version::parse("1.0.0").unwrap() > Version::parse("0.9.9").unwrap());
        assert!(Version::parse("1.1.0").unwrap() > Version::parse("1.0.9").unwrap());
        assert!(Version::parse("1.0.1").unwrap() > Version::parse("1.0.0").unwrap());
    }
}
