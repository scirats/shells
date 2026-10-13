use std::fmt::{self, Write};

use text::Template;

pub(crate) struct YamlInput<'a> {
    pub image: &'a str,
    pub cpus: u32,
    pub memory: &'a str,
    pub disk: &'a str,
    pub mounts: &'a str,
}

pub(crate) struct MountSpec {
    pub location: String,
    pub mount_point: Option<String>,
    pub writable: bool,
}

pub(crate) fn mounts_block(mounts: &[MountSpec]) -> String {
    let mut out = String::new();
    write_line(&mut out, format_args!("mounts:"));
    for mount in mounts {
        write_line(
            &mut out,
            format_args!("  - location: {}", yaml_quote(&mount.location)),
        );
        if let Some(point) = &mount.mount_point {
            write_line(
                &mut out,
                format_args!("    mountPoint: {}", yaml_quote(point)),
            );
        }
        write_line(&mut out, format_args!("    writable: {}", mount.writable));
    }
    out
}

fn write_line(out: &mut String, args: fmt::Arguments<'_>) {
    out.write_fmt(args)
        .expect("writing into a String is infallible");
    out.push('\n');
}

fn yaml_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

pub(crate) fn lima_yaml(template: &Template, input: &YamlInput<'_>) -> String {
    let cpus = input.cpus.to_string();
    template.render(&[
        ("image", input.image),
        ("cpus", &cpus),
        ("memory", input.memory),
        ("disk", input.disk),
        ("mounts", input.mounts),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEMPLATE: &str = include_str!("../../../templates/lima.yaml");

    fn input() -> YamlInput<'static> {
        YamlInput {
            image: "ubuntu",
            cpus: 4,
            memory: "4GiB",
            disk: "40GiB",
            mounts: "mounts: []",
        }
    }

    #[test]
    fn renders_core_fields() {
        let out = lima_yaml(&Template::new(TEMPLATE), &input());
        assert!(out.contains("base: template:ubuntu"));
        assert!(out.contains("cpus: 4"));
    }

    #[test]
    fn leaves_no_placeholders() {
        let out = lima_yaml(&Template::new(TEMPLATE), &input());
        assert!(!out.contains("{{"));
    }

    #[test]
    fn renders_mounts_with_mount_point() {
        let mounts = vec![
            MountSpec {
                location: "/host/project".to_string(),
                mount_point: Some("{{.Home}}/Workspace".to_string()),
                writable: true,
            },
            MountSpec {
                location: "/host/ws".to_string(),
                mount_point: None,
                writable: false,
            },
        ];
        let block = mounts_block(&mounts);
        assert!(block.contains("mountPoint: \"{{.Home}}/Workspace\""));
        assert!(block.contains("writable: false"));
    }
}
