use comfy_table::{presets::NOTHING, Table};
use kernel::{Profile, ShellService, ShellSummary, REQUIRED_PACKAGES};

pub(crate) fn table(shells: &[ShellSummary]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(NOTHING)
        .set_header(vec!["NAME", "STATUS", "IMAGE", "WORKSPACE"]);
    for shell in shells {
        table.add_row(vec![
            shell.name.to_string(),
            shell.phase.label(),
            shell.profile.image.clone(),
            workspace(&shell.profile),
        ]);
    }
    flush_edges(&mut table);
    table
}

pub(crate) fn detail(shell: &ShellSummary, service: &ShellService) -> Table {
    let profile = &shell.profile;
    let rows = vec![
        ("NAME".to_string(), shell.name.to_string()),
        ("STATUS".to_string(), shell.phase.label()),
        ("IMAGE".to_string(), profile.image.clone()),
        ("CPUS".to_string(), profile.cpus.to_string()),
        ("MEMORY".to_string(), profile.memory.clone()),
        ("DISK".to_string(), profile.disk.clone()),
        ("WORKSPACE".to_string(), workspace(profile)),
        ("PACKAGES".to_string(), REQUIRED_PACKAGES.join(", ")),
        ("DOTFILES".to_string(), service.dotfiles_dir(&shell.name)),
        ("SHELL DIR".to_string(), service.shell_dir(&shell.name)),
    ];

    let mut table = Table::new();
    table
        .load_preset(NOTHING)
        .set_header(vec!["FIELD", "VALUE"]);
    for (key, value) in rows {
        table.add_row(vec![key, value]);
    }
    flush_edges(&mut table);
    table
}

fn flush_edges(table: &mut Table) {
    let last = table.column_count().saturating_sub(1);
    if let Some(column) = table.column_mut(0) {
        column.set_padding((0, 1));
    }
    if let Some(column) = table.column_mut(last) {
        column.set_padding((1, 0));
    }
}

fn workspace(profile: &Profile) -> String {
    profile.workspace.clone().unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{ShellName, VmPhase};

    fn summary(name: &str, phase: VmPhase, image: &str, workspace: Option<&str>) -> ShellSummary {
        ShellSummary {
            name: ShellName::parse(name).unwrap(),
            profile: Profile {
                image: image.to_string(),
                workspace: workspace.map(str::to_string),
                ..Profile::default()
            },
            phase,
        }
    }

    #[test]
    fn summary_is_borderless_and_aligned() {
        let shells = vec![
            summary(
                "demo",
                VmPhase::Running,
                "debian",
                Some("/Users/me/Workspaces/project"),
            ),
            summary("web", VmPhase::Stopped, "ubuntu", None),
        ];
        let rendered = table(&shells).to_string();
        println!("[[[{rendered}]]]");
        assert!(!rendered.contains(['┌', '│', '─']));
        assert!(rendered.lines().next().unwrap().contains("NAME"));
    }
}
