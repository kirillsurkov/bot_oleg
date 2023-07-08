pub struct Formatter {
    first: String,
    second: String,
}

impl std::str::FromStr for Formatter {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (first, second) = s.split_once("{}").ok_or("no `{}` in format string")?;
        if second.contains("{}") {
            return Err("more than one `{}` in format string");
        }
        Ok(Self {
            first: first.to_owned(),
            second: second.to_owned(),
        })
    }
}

impl Formatter {
    pub fn format(&self, value: &impl std::fmt::Display) -> String {
        let Self { first, second } = self;
        format!("{first}{value}{second}")
    }
}
