use kernel::{VmError, VmPhase};

#[derive(Debug, Clone)]
pub(crate) struct Instance {
    pub name: String,
    pub status: String,
}

pub(crate) fn phase_from_status(status: &str) -> VmPhase {
    match status {
        "Running" => VmPhase::Running,
        "Stopped" => VmPhase::Stopped,
        other => VmPhase::Unknown(other.to_string()),
    }
}

pub(crate) fn phase_of(instances: &[Instance], name: &str) -> VmPhase {
    instances
        .iter()
        .find(|instance| instance.name == name)
        .map(|instance| phase_from_status(&instance.status))
        .unwrap_or(VmPhase::NotCreated)
}

pub(crate) fn parse_instances(text: &str) -> Result<Vec<Instance>, VmError> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    if text.starts_with('[') {
        parse_array(text)
    } else {
        parse_lines(text)
    }
}

fn parse_array(text: &str) -> Result<Vec<Instance>, VmError> {
    let values: Vec<serde_json::Value> = serde_json::from_str(text).map_err(parse_error)?;
    Ok(values.iter().filter_map(read_instance).collect())
}

fn parse_lines(text: &str) -> Result<Vec<Instance>, VmError> {
    let mut instances = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let value: serde_json::Value = serde_json::from_str(line).map_err(parse_error)?;
        if let Some(instance) = read_instance(&value) {
            instances.push(instance);
        }
    }
    Ok(instances)
}

fn read_instance(value: &serde_json::Value) -> Option<Instance> {
    let name = value.get("name")?.as_str()?.to_string();
    let status = value
        .get("status")
        .and_then(|value| value.as_str())
        .unwrap_or("Unknown")
        .to_string();
    Some(Instance { name, status })
}

fn parse_error(error: serde_json::Error) -> VmError {
    VmError::Internal(format!("cannot parse limactl list: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ndjson_instances() {
        let text =
            "{\"name\":\"a\",\"status\":\"Running\"}\n{\"name\":\"b\",\"status\":\"Stopped\"}";
        let instances = parse_instances(text).unwrap();
        assert_eq!(instances.len(), 2);
        assert_eq!(instances[0].name, "a");
        assert_eq!(phase_from_status(&instances[1].status), VmPhase::Stopped);
    }

    #[test]
    fn parses_array_instances() {
        let text = r#"[{"name":"a","status":"Running"}]"#;
        let instances = parse_instances(text).unwrap();
        assert_eq!(instances.len(), 1);
        assert_eq!(phase_from_status(&instances[0].status), VmPhase::Running);
    }

    #[test]
    fn parses_empty_output() {
        assert!(parse_instances("").unwrap().is_empty());
    }
}
