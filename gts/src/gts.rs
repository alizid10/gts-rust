use regex::Regex;
use std::sync::LazyLock;
use thiserror::Error;
use uuid::Uuid;

pub const GTS_PREFIX: &str = "gts.";
static GTS_NS: LazyLock<Uuid> = LazyLock::new(|| Uuid::new_v5(&Uuid::NAMESPACE_URL, b"gts"));
static GTS_SEGMENT_TOKEN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z_][a-z0-9_]*$").unwrap());

#[derive(Debug, Error)]
pub enum GtsError {
    #[error("Invalid GTS segment #{num} @ offset {offset}: '{segment}': {cause}")]
    InvalidSegment {
        num: usize,
        offset: usize,
        segment: String,
        cause: String,
    },

    #[error("Invalid GTS identifier: {id}: {cause}")]
    InvalidId { id: String, cause: String },

    #[error("Invalid GTS wildcard pattern: {pattern}: {cause}")]
    InvalidWildcard { pattern: String, cause: String },
}

/// Parsed GTS segment
#[derive(Debug, Clone, PartialEq)]
pub struct GtsIdSegment {
    pub num: usize,
    pub offset: usize,
    pub segment: String,
    pub vendor: String,
    pub package: String,
    pub namespace: String,
    pub type_name: String,
    pub ver_major: u32,
    pub ver_minor: Option<u32>,
    pub is_type: bool,
    pub is_wildcard: bool,
}

impl GtsIdSegment {
    pub fn new(num: usize, offset: usize, segment: &str) -> Result<Self, GtsError> {
        let segment = segment.trim().to_string();
        let mut seg = GtsIdSegment {
            num,
            offset,
            segment: segment.clone(),
            vendor: String::new(),
            package: String::new(),
            namespace: String::new(),
            type_name: String::new(),
            ver_major: 0,
            ver_minor: None,
            is_type: false,
            is_wildcard: false,
        };

        seg.parse_segment_id(&segment)?;
        Ok(seg)
    }

    fn parse_segment_id(&mut self, segment: &str) -> Result<(), GtsError> {
        let mut segment = segment.to_string();

        // Check for type marker
        if segment.contains('~') {
            let tilde_count = segment.matches('~').count();
            if tilde_count > 1 {
                return Err(GtsError::InvalidSegment {
                    num: self.num,
                    offset: self.offset,
                    segment: self.segment.clone(),
                    cause: "Too many '~' characters".to_string(),
                });
            }
            if segment.ends_with('~') {
                self.is_type = true;
                segment.pop();
            } else {
                return Err(GtsError::InvalidSegment {
                    num: self.num,
                    offset: self.offset,
                    segment: self.segment.clone(),
                    cause: " '~' must be at the end".to_string(),
                });
            }
        }

        let tokens: Vec<&str> = segment.split('.').collect();

        if tokens.len() > 6 {
            return Err(GtsError::InvalidSegment {
                num: self.num,
                offset: self.offset,
                segment: self.segment.clone(),
                cause: "Too many tokens".to_string(),
            });
        }

        if !segment.ends_with('*') && tokens.len() < 5 {
            return Err(GtsError::InvalidSegment {
                num: self.num,
                offset: self.offset,
                segment: self.segment.clone(),
                cause: "Too few tokens".to_string(),
            });
        }

        // Validate tokens (except version tokens)
        if !segment.ends_with('*') {
            for i in 0..4 {
                if !GTS_SEGMENT_TOKEN_REGEX.is_match(tokens[i]) {
                    return Err(GtsError::InvalidSegment {
                        num: self.num,
                        offset: self.offset,
                        segment: self.segment.clone(),
                        cause: format!("Invalid segment token: {}", tokens[i]),
                    });
                }
            }
        }

        // Parse tokens
        if !tokens.is_empty() {
            if tokens[0] == "*" {
                self.is_wildcard = true;
                return Ok(());
            }
            self.vendor = tokens[0].to_string();
        }

        if tokens.len() > 1 {
            if tokens[1] == "*" {
                self.is_wildcard = true;
                return Ok(());
            }
            self.package = tokens[1].to_string();
        }

        if tokens.len() > 2 {
            if tokens[2] == "*" {
                self.is_wildcard = true;
                return Ok(());
            }
            self.namespace = tokens[2].to_string();
        }

        if tokens.len() > 3 {
            if tokens[3] == "*" {
                self.is_wildcard = true;
                return Ok(());
            }
            self.type_name = tokens[3].to_string();
        }

        if tokens.len() > 4 {
            if tokens[4] == "*" {
                self.is_wildcard = true;
                return Ok(());
            }

            if !tokens[4].starts_with('v') {
                return Err(GtsError::InvalidSegment {
                    num: self.num,
                    offset: self.offset,
                    segment: self.segment.clone(),
                    cause: "Major version must start with 'v'".to_string(),
                });
            }

            let major_str = &tokens[4][1..];
            self.ver_major = major_str.parse().map_err(|_| GtsError::InvalidSegment {
                num: self.num,
                offset: self.offset,
                segment: self.segment.clone(),
                cause: "Major version must be an integer".to_string(),
            })?;

            if major_str != self.ver_major.to_string() {
                return Err(GtsError::InvalidSegment {
                    num: self.num,
                    offset: self.offset,
                    segment: self.segment.clone(),
                    cause: "Major version must be an integer".to_string(),
                });
            }
        }

        if tokens.len() > 5 {
            if tokens[5] == "*" {
                self.is_wildcard = true;
                return Ok(());
            }

            let minor: u32 = tokens[5].parse().map_err(|_| GtsError::InvalidSegment {
                num: self.num,
                offset: self.offset,
                segment: self.segment.clone(),
                cause: "Minor version must be an integer".to_string(),
            })?;

            if tokens[5] != minor.to_string() {
                return Err(GtsError::InvalidSegment {
                    num: self.num,
                    offset: self.offset,
                    segment: self.segment.clone(),
                    cause: "Minor version must be an integer".to_string(),
                });
            }

            self.ver_minor = Some(minor);
        }

        Ok(())
    }
}

/// GTS ID
#[derive(Debug, Clone, PartialEq)]
pub struct GtsID {
    pub id: String,
    pub gts_id_segments: Vec<GtsIdSegment>,
}

impl GtsID {
    pub fn new(id: &str) -> Result<Self, GtsError> {
        let raw = id.trim();

        // Validate lowercase
        if raw != raw.to_lowercase() {
            return Err(GtsError::InvalidId {
                id: id.to_string(),
                cause: "Must be lower case".to_string(),
            });
        }

        if raw.contains('-') {
            return Err(GtsError::InvalidId {
                id: id.to_string(),
                cause: "Must not contain '-'".to_string(),
            });
        }

        if !raw.starts_with(GTS_PREFIX) {
            return Err(GtsError::InvalidId {
                id: id.to_string(),
                cause: format!("Does not start with '{}'", GTS_PREFIX),
            });
        }

        if raw.len() > 1024 {
            return Err(GtsError::InvalidId {
                id: id.to_string(),
                cause: "Too long".to_string(),
            });
        }

        let mut gts_id_segments = Vec::new();
        let remainder = &raw[GTS_PREFIX.len()..];

        // Split by ~ preserving empties to detect trailing ~
        let _parts: Vec<&str> = remainder.split('~').collect();
        let mut parts = Vec::new();

        for i in 0.._parts.len() {
            if i < _parts.len() - 1 {
                parts.push(format!("{}~", _parts[i]));
                if i == _parts.len() - 2 && _parts[i + 1].is_empty() {
                    break;
                }
            } else {
                parts.push(_parts[i].to_string());
            }
        }

        let mut offset = GTS_PREFIX.len();
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() || part == "~" {
                return Err(GtsError::InvalidId {
                    id: id.to_string(),
                    cause: format!("GTS segment #{} @ offset {} is empty", i + 1, offset),
                });
            }

            gts_id_segments.push(GtsIdSegment::new(i + 1, offset, part)?);
            offset += part.len();
        }

        Ok(GtsID {
            id: raw.to_string(),
            gts_id_segments,
        })
    }

    pub fn is_type(&self) -> bool {
        self.id.ends_with('~')
    }

    pub fn get_type_id(&self) -> Option<String> {
        if self.gts_id_segments.len() < 2 {
            return None;
        }
        let segments: String = self.gts_id_segments[..self.gts_id_segments.len() - 1]
            .iter()
            .map(|s| s.segment.as_str())
            .collect::<Vec<_>>()
            .join("");
        Some(format!("{}{}", GTS_PREFIX, segments))
    }

    pub fn to_uuid(&self) -> Uuid {
        Uuid::new_v5(&GTS_NS, self.id.as_bytes())
    }

    pub fn is_valid(s: &str) -> bool {
        if !s.starts_with(GTS_PREFIX) {
            return false;
        }
        Self::new(s).is_ok()
    }

    pub fn wildcard_match(&self, pattern: &GtsWildcard) -> bool {
        let p = &pattern.id;

        // No wildcard case - need exact match with version flexibility
        if !p.contains('*') {
            return self.match_segments(&pattern.gts_id_segments, &self.gts_id_segments);
        }

        // Wildcard case
        if p.matches('*').count() > 1 || !p.ends_with('*') {
            return false;
        }

        self.match_segments(&pattern.gts_id_segments, &self.gts_id_segments)
    }

    fn match_segments(
        &self,
        pattern_segs: &[GtsIdSegment],
        candidate_segs: &[GtsIdSegment],
    ) -> bool {
        // If pattern is longer than candidate, no match
        if pattern_segs.len() > candidate_segs.len() {
            return false;
        }

        for (i, p_seg) in pattern_segs.iter().enumerate() {
            let c_seg = &candidate_segs[i];

            // If pattern segment is a wildcard, check non-wildcard fields first
            if p_seg.is_wildcard {
                if !p_seg.vendor.is_empty() && p_seg.vendor != c_seg.vendor {
                    return false;
                }
                if !p_seg.package.is_empty() && p_seg.package != c_seg.package {
                    return false;
                }
                if !p_seg.namespace.is_empty() && p_seg.namespace != c_seg.namespace {
                    return false;
                }
                if !p_seg.type_name.is_empty() && p_seg.type_name != c_seg.type_name {
                    return false;
                }
                if p_seg.ver_major != 0 && p_seg.ver_major != c_seg.ver_major {
                    return false;
                }
                if let Some(p_minor) = p_seg.ver_minor {
                    if Some(p_minor) != c_seg.ver_minor {
                        return false;
                    }
                }
                if p_seg.is_type && p_seg.is_type != c_seg.is_type {
                    return false;
                }
                // Wildcard matches - accept anything after this point
                return true;
            }

            // Non-wildcard segment - all fields must match exactly
            if p_seg.vendor != c_seg.vendor {
                return false;
            }
            if p_seg.package != c_seg.package {
                return false;
            }
            if p_seg.namespace != c_seg.namespace {
                return false;
            }
            if p_seg.type_name != c_seg.type_name {
                return false;
            }

            // Check version matching
            if p_seg.ver_major != c_seg.ver_major {
                return false;
            }

            // Minor version: if pattern has no minor version, accept any minor in candidate
            if let Some(p_minor) = p_seg.ver_minor {
                if Some(p_minor) != c_seg.ver_minor {
                    return false;
                }
            }

            // Check is_type flag matches
            if p_seg.is_type != c_seg.is_type {
                return false;
            }
        }

        true
    }

    pub fn split_at_path(gts_with_path: &str) -> Result<(String, Option<String>), GtsError> {
        if !gts_with_path.contains('@') {
            return Ok((gts_with_path.to_string(), None));
        }

        let parts: Vec<&str> = gts_with_path.splitn(2, '@').collect();
        let gts = parts[0].to_string();
        let path = parts.get(1).map(|s| s.to_string());

        if let Some(ref p) = path {
            if p.is_empty() {
                return Err(GtsError::InvalidId {
                    id: gts_with_path.to_string(),
                    cause: "Attribute path cannot be empty".to_string(),
                });
            }
        }

        Ok((gts, path))
    }
}

/// GTS Wildcard pattern
#[derive(Debug, Clone, PartialEq)]
pub struct GtsWildcard {
    pub id: String,
    pub gts_id_segments: Vec<GtsIdSegment>,
}

impl GtsWildcard {
    pub fn new(pattern: &str) -> Result<Self, GtsError> {
        let p = pattern.trim();

        if !p.starts_with(GTS_PREFIX) {
            return Err(GtsError::InvalidWildcard {
                pattern: pattern.to_string(),
                cause: format!("Does not start with '{}'", GTS_PREFIX),
            });
        }

        if p.matches('*').count() > 1 {
            return Err(GtsError::InvalidWildcard {
                pattern: pattern.to_string(),
                cause: "The wildcard '*' token is allowed only once".to_string(),
            });
        }

        if p.contains('*') && !p.ends_with(".*") && !p.ends_with("~*") {
            return Err(GtsError::InvalidWildcard {
                pattern: pattern.to_string(),
                cause: "The wildcard '*' token is allowed only at the end of the pattern"
                    .to_string(),
            });
        }

        // Try to parse as GtsID
        let gts_id = GtsID::new(p).map_err(|e| GtsError::InvalidWildcard {
            pattern: pattern.to_string(),
            cause: e.to_string(),
        })?;

        Ok(GtsWildcard {
            id: gts_id.id,
            gts_id_segments: gts_id.gts_id_segments,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gts_id_valid() {
        let id = GtsID::new("gts.x.core.events.event.v1~").unwrap();
        assert_eq!(id.id, "gts.x.core.events.event.v1~");
        assert!(id.is_type());
    }

    #[test]
    fn test_gts_id_invalid_uppercase() {
        let result = GtsID::new("gts.X.core.events.event.v1~");
        assert!(result.is_err());
    }

    #[test]
    fn test_gts_wildcard() {
        let pattern = GtsWildcard::new("gts.x.core.events.*").unwrap();
        let id = GtsID::new("gts.x.core.events.event.v1~").unwrap();
        assert!(id.wildcard_match(&pattern));
    }

    #[test]
    fn test_uuid_generation() {
        let id = GtsID::new("gts.x.core.events.event.v1~").unwrap();
        let uuid = id.to_uuid();
        assert!(!uuid.to_string().is_empty());
    }
}
