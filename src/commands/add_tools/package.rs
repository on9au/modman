use crate::datatypes::ModSources;

#[derive(Debug)]
pub struct Package {
    pub search_term: String,
    pub source: ModSources,
}

impl Package {
    // A constructor to create a new Package instance.
    pub fn new(search_term: String, source: Option<&str>) -> Result<Self, String> {
        let source: ModSources = match source {
            Some(s) => s.parse::<crate::datatypes::ModSources>()?,
            None => ModSources::Modrinth, // Default to Modrinth if no source is provided
        };
        Ok(Package { search_term, source })
    }
}