use std::fmt::Display;

pub struct BepInExMod {
    name: String,
    version: String,
}

impl BepInExMod {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl Display for BepInExMod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} {}", self.name, self.version))
    }
}
