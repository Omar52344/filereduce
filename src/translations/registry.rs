use crate::error::{FileReduceError, Result};
use crate::translations::config::*;
use std::fs;
use std::path::Path;
use std::sync::RwLock;

#[derive(Debug)]
pub struct TranslationRegistry {
    config: RwLock<TranslationConfig>,
}

impl TranslationRegistry {
    pub fn new() -> Result<Self> {
        let config = Self::load_default()?;
        Ok(Self {
            config: RwLock::new(config),
        })
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: TranslationConfig =
            serde_json::from_str(&content).map_err(|e| FileReduceError::Serialization(e))?;
        Ok(Self {
            config: RwLock::new(config),
        })
    }

    fn load_default() -> Result<TranslationConfig> {
        let content = include_str!("../../translations.json");
        serde_json::from_str(content).map_err(|e| FileReduceError::Serialization(e))
    }

    pub fn get_segment(&self, segment_code: &str) -> Option<SegmentConfig> {
        let config = self.config.read().unwrap();
        config.segments.get(segment_code).cloned()
    }

    pub fn get_qualifier(&self, segment_code: &str, qualifier: &str) -> Option<SubSegmentConfig> {
        let config = self.config.read().unwrap();
        config.segments.get(segment_code).and_then(|seg| {
            if seg.use_qualifier {
                seg.qualifiers.get(qualifier).cloned()
            } else {
                None
            }
        })
    }

    pub fn get_element(&self, segment_code: &str, element_pos: &str) -> Option<ElementConfig> {
        let config = self.config.read().unwrap();
        config
            .segments
            .get(segment_code)
            .and_then(|seg| seg.elements.get(element_pos).cloned())
    }

    pub fn reload_from_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let new_config: TranslationConfig =
            serde_json::from_str(&content).map_err(|e| FileReduceError::Serialization(e))?;
        let mut config = self.config.write().unwrap();
        *config = new_config;
        Ok(())
    }
}

impl Clone for TranslationRegistry {
    fn clone(&self) -> Self {
        let config = self.config.read().unwrap().clone();
        Self {
            config: RwLock::new(config),
        }
    }
}

impl Default for TranslationRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to load default translation config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default() {
        let registry = TranslationRegistry::new();
        assert!(registry.is_ok());
        let registry = registry.unwrap();

        // Check that some expected segments are loaded
        let bgm = registry.get_segment("BGM");
        assert!(bgm.is_some());
        let bgm_config = bgm.unwrap();
        assert_eq!(bgm_config.label, "DocumentHeader");
        assert!(bgm_config.elements.contains_key("1"));

        // Check qualifier segments
        let dtm = registry.get_segment("DTM");
        assert!(dtm.is_some());
        let dtm_config = dtm.unwrap();
        assert!(dtm_config.use_qualifier);
        assert!(dtm_config.qualifiers.contains_key("137"));

        // Check element mapping
        let element = registry.get_element("BGM", "1");
        assert!(matches!(element, Some(ElementConfig::Simple(_))));
    }
}
