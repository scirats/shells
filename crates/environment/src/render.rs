use kernel::{Shell, GUEST_STAGE_DIR};
use text::Template;

pub(crate) fn env_script(template: &Template, shell: &Shell) -> String {
    template.render(&[("name", shell.name.as_str())])
}

pub(crate) fn apply_script(template: &Template, browser_port: u16, term: &str) -> String {
    let port = browser_port.to_string();
    template.render(&[
        ("source", GUEST_STAGE_DIR),
        ("browser_port", &port),
        ("term", term),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{Profile, ShellName};

    const ENV: &str = include_str!("../../../templates/env.sh");
    const APPLY: &str = include_str!("../../../templates/apply.sh");
    const BROWSER_PORT: u16 = 2718;

    fn shell() -> Shell {
        Shell {
            name: ShellName::parse("alpha").unwrap(),
            profile: Profile::default(),
        }
    }

    #[test]
    fn env_script_exports_the_shell_name() {
        let out = env_script(&Template::new(ENV), &shell());
        assert!(out.contains("SHELL_MANAGER_NAME=\"alpha\""));
        assert!(!out.contains("{{"));
    }

    #[test]
    fn apply_script_embeds_the_stage_source_browser_port_and_term() {
        let out = apply_script(&Template::new(APPLY), BROWSER_PORT, "xterm-kitty");
        assert!(out.contains("SRC=\"/tmp/shell\""));
        assert!(out.contains(&format!("127.0.0.1:{BROWSER_PORT}/open")));
        assert!(out.contains("infocmp \"xterm-kitty\""));
        assert!(!out.contains("{{"));
    }
}
