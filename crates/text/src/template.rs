use std::path::Path;

pub struct Template(String);

impl Template {
    pub fn new(source: impl Into<String>) -> Self {
        Self(source.into())
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        std::fs::read_to_string(path).map(Self)
    }

    pub fn render(&self, values: &[(&str, &str)]) -> String {
        let mut out = String::with_capacity(self.0.len());
        for line in self.0.split_inclusive('\n') {
            let body = line.strip_suffix('\n').unwrap_or(line);
            if let Some(key) = sole_placeholder(body) {
                match value(values, key) {
                    Some("") => {}
                    Some(replacement) => {
                        out.push_str(replacement);
                        if !replacement.ends_with('\n') {
                            out.push('\n');
                        }
                    }
                    None => out.push_str(line),
                }
            } else {
                out.push_str(&interpolate(body, values));
                if line.ends_with('\n') {
                    out.push('\n');
                }
            }
        }
        out
    }
}

fn sole_placeholder(line: &str) -> Option<&str> {
    let key = line.strip_prefix("{{")?.strip_suffix("}}")?;
    let is_word = !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    is_word.then_some(key)
}

fn interpolate(line: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(line.len());
    let mut cursor = 0;
    while let Some(offset) = line[cursor..].find("{{") {
        let open = cursor + offset;
        out.push_str(&line[cursor..open]);
        let body = open + 2;
        match line[body..].find("}}") {
            Some(offset) => {
                let close = body + offset;
                match value(values, &line[body..close]) {
                    Some(replacement) => out.push_str(replacement),
                    None => out.push_str(&line[open..close + 2]),
                }
                cursor = close + 2;
            }
            None => {
                out.push_str(&line[open..]);
                return out;
            }
        }
    }
    out.push_str(&line[cursor..]);
    out
}

fn value<'a>(values: &[(&str, &'a str)], key: &str) -> Option<&'a str> {
    values
        .iter()
        .find(|(name, _)| *name == key)
        .map(|(_, value)| *value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_each_inline_placeholder() {
        let template = Template::new("a {{x}} b {{x}} c {{y}}");
        assert_eq!(template.render(&[("x", "1"), ("y", "2")]), "a 1 b 1 c 2");
    }

    #[test]
    fn keeps_unknown_placeholders() {
        let template = Template::new("{{known}} and {{unknown}}");
        assert_eq!(template.render(&[("known", "yes")]), "yes and {{unknown}}");
    }

    #[test]
    fn does_not_reprocess_inserted_values() {
        let template = Template::new("{{a}}{{b}}");
        assert_eq!(template.render(&[("a", "{{b}}"), ("b", "x")]), "{{b}}x");
    }

    #[test]
    fn leaves_shell_braces_untouched() {
        let template = Template::new("${VAR} { block }");
        assert_eq!(template.render(&[("x", "1")]), "${VAR} { block }");
    }

    #[test]
    fn drops_a_sole_placeholder_line_when_empty() {
        let template = Template::new("a\n{{x}}\nb\n");
        assert_eq!(template.render(&[("x", "")]), "a\nb\n");
    }

    #[test]
    fn expands_a_sole_placeholder_line() {
        let template = Template::new("a\n{{x}}\nb\n");
        assert_eq!(template.render(&[("x", "1\n2\n")]), "a\n1\n2\nb\n");
    }

    #[test]
    fn terminates_a_sole_placeholder_value_without_newline() {
        let template = Template::new("a\n{{x}}\nb\n");
        assert_eq!(template.render(&[("x", "1")]), "a\n1\nb\n");
    }
}
