use kernel::{Profile, StoreError};

use crate::internal;

pub(crate) fn encode(profile: &Profile) -> Result<String, StoreError> {
    serde_json::to_string(profile).map_err(internal)
}

pub(crate) fn decode(json: &str) -> Result<Profile, StoreError> {
    serde_json::from_str(json).map_err(internal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::Profile;

    #[test]
    fn round_trips_a_profile() {
        let profile = Profile::default();
        let decoded = decode(&encode(&profile).unwrap()).unwrap();
        assert_eq!(decoded.image, profile.image);
        assert_eq!(decoded.cpus, profile.cpus);
        assert_eq!(decoded.workspace, profile.workspace);
    }
}
