//! What an Android export needs besides the extension library.

/// Maven artifacts the Android Gradle build needs, `group:artifact:version`.
///
/// ```
/// use hqf_godot_google_signin::android::ANDROID_DEPENDENCIES;
///
/// assert!(ANDROID_DEPENDENCIES.contains(&"androidx.credentials:credentials:1.6.0"));
/// ```
pub const ANDROID_DEPENDENCIES: [&str; 3] = [
    "androidx.credentials:credentials:1.6.0",
    "androidx.credentials:credentials-play-services-auth:1.6.0",
    "com.google.android.libraries.identity.googleid:googleid:1.2.1",
];

#[cfg(test)]
mod tests {
    use super::ANDROID_DEPENDENCIES;

    #[test]
    fn every_dependency_is_group_artifact_version() {
        for dependency in ANDROID_DEPENDENCIES {
            let parts: Vec<&str> = dependency.split(':').collect();
            assert_eq!(parts.len(), 3, "{dependency}");
            assert!(parts.iter().all(|part| !part.is_empty()), "{dependency}");
        }
    }

    #[test]
    fn credentials_and_play_services_share_a_version() {
        let version = |artifact: &str| {
            ANDROID_DEPENDENCIES
                .iter()
                .find(|dependency| dependency.contains(artifact))
                .and_then(|dependency| dependency.rsplit(':').next())
                .unwrap()
        };
        assert_eq!(
            version(":credentials:"),
            version(":credentials-play-services-auth:")
        );
    }
}
