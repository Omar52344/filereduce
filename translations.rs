#[derive(Deserialize, Debug)]
struct TranslationConfig {
    version: String,
    segments: HashMap<String, SegmentConfig>,
}

#[derive(Deserialize, Debug)]
struct SegmentConfig {
    label: String,
    #[serde(default)]
    use_qualifier: bool,
    #[serde(default)]
    qualifiers: HashMap<String, SubSegmentConfig>,
    #[serde(default)]
    elements: HashMap<String, ElementConfig>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ElementConfig {
    Simple(String),
    Composite {
        label: String,
        components: HashMap<String, String>,
    },
}