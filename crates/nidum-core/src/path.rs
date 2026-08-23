/// Combines a base controller path and a handler subpath into a clean, normalized route path.
///
/// # Examples
/// ```
/// use nidum_core::path::combine_paths;
///
/// assert_eq!(combine_paths("", "/"), "/");
/// assert_eq!(combine_paths("/users", "/"), "/users");
/// assert_eq!(combine_paths("/users", ""), "/users");
/// assert_eq!(combine_paths("/users", "/:id"), "/users/:id");
/// assert_eq!(combine_paths("users", "profile"), "/users/profile");
/// ```
pub fn combine_paths(base: &str, sub: &str) -> String {
    let base = base.trim_matches('/');
    let sub = sub.trim_matches('/');

    match (base.is_empty(), sub.is_empty()) {
        (true, true) => "/".to_string(),
        (false, true) => format!("/{}", base),
        (true, false) => format!("/{}", sub),
        (false, false) => format!("/{}/{}", base, sub),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_paths() {
        assert_eq!(combine_paths("", ""), "/");
        assert_eq!(combine_paths("", "/"), "/");
        assert_eq!(combine_paths("/", "/"), "/");
        assert_eq!(combine_paths("/api", "/"), "/api");
        assert_eq!(combine_paths("/users", "/:id"), "/users/:id");
        assert_eq!(combine_paths("users", "create"), "/users/create");
    }
}
