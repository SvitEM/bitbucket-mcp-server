use crate::types::BitbucketConfig;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Read,
    Write,
    Delete,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operation::Read => write!(f, "read"),
            Operation::Write => write!(f, "write"),
            Operation::Delete => write!(f, "delete"),
        }
    }
}

#[derive(Debug)]
pub enum PermissionError {
    OperationDenied {
        operation: Operation,
        message: String,
    },
}

impl fmt::Display for PermissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionError::OperationDenied { operation, message } => {
                write!(f, "Permission denied for {} operation: {}", operation, message)
            }
        }
    }
}

impl std::error::Error for PermissionError {}

pub struct PermissionChecker {
    allow_read: bool,
    allow_write: bool,
    allow_delete: bool,
}

impl PermissionChecker {
    pub fn new(config: &BitbucketConfig) -> Self {
        Self {
            allow_read: config.allow_read,
            allow_write: config.allow_write,
            allow_delete: config.allow_delete,
        }
    }

    pub fn check_permission(&self, operation: Operation) -> Result<(), PermissionError> {
        let allowed = match operation {
            Operation::Read => self.allow_read,
            Operation::Write => self.allow_write,
            Operation::Delete => self.allow_delete,
        };

        if allowed {
            Ok(())
        } else {
            Err(PermissionError::OperationDenied {
                operation,
                message: format!(
                    "{} operations are disabled by configuration",
                    operation
                ),
            })
        }
    }

    pub fn can_read(&self) -> bool {
        self.allow_read
    }

    pub fn can_write(&self) -> bool {
        self.allow_write
    }

    pub fn can_delete(&self) -> bool {
        self.allow_delete
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BitbucketConfig;

    fn create_test_config(allow_read: bool, allow_write: bool, allow_delete: bool) -> BitbucketConfig {
        BitbucketConfig {
            base_url: "https://test.com".to_string(),
            username: "test".to_string(),
            password: "test".to_string(),
            verify_ssl: true,
            allow_read,
            allow_write,
            allow_delete,
        }
    }

    #[test]
    fn test_all_permissions_allowed() {
        let config = create_test_config(true, true, true);
        let checker = PermissionChecker::new(&config);

        assert!(checker.can_read());
        assert!(checker.can_write());
        assert!(checker.can_delete());
        assert!(checker.check_permission(Operation::Read).is_ok());
        assert!(checker.check_permission(Operation::Write).is_ok());
        assert!(checker.check_permission(Operation::Delete).is_ok());
    }

    #[test]
    fn test_all_permissions_denied() {
        let config = create_test_config(false, false, false);
        let checker = PermissionChecker::new(&config);

        assert!(!checker.can_read());
        assert!(!checker.can_write());
        assert!(!checker.can_delete());
        assert!(checker.check_permission(Operation::Read).is_err());
        assert!(checker.check_permission(Operation::Write).is_err());
        assert!(checker.check_permission(Operation::Delete).is_err());
    }

    #[test]
    fn test_read_only_permissions() {
        let config = create_test_config(true, false, false);
        let checker = PermissionChecker::new(&config);

        assert!(checker.can_read());
        assert!(!checker.can_write());
        assert!(!checker.can_delete());
        assert!(checker.check_permission(Operation::Read).is_ok());
        assert!(checker.check_permission(Operation::Write).is_err());
        assert!(checker.check_permission(Operation::Delete).is_err());
    }

    #[test]
    fn test_write_only_permissions() {
        let config = create_test_config(false, true, false);
        let checker = PermissionChecker::new(&config);

        assert!(!checker.can_read());
        assert!(checker.can_write());
        assert!(!checker.can_delete());
        assert!(checker.check_permission(Operation::Read).is_err());
        assert!(checker.check_permission(Operation::Write).is_ok());
        assert!(checker.check_permission(Operation::Delete).is_err());
    }

    #[test]
    fn test_delete_only_permissions() {
        let config = create_test_config(false, false, true);
        let checker = PermissionChecker::new(&config);

        assert!(!checker.can_read());
        assert!(!checker.can_write());
        assert!(checker.can_delete());
        assert!(checker.check_permission(Operation::Read).is_err());
        assert!(checker.check_permission(Operation::Write).is_err());
        assert!(checker.check_permission(Operation::Delete).is_ok());
    }

    #[test]
    fn test_read_write_permissions() {
        let config = create_test_config(true, true, false);
        let checker = PermissionChecker::new(&config);

        assert!(checker.can_read());
        assert!(checker.can_write());
        assert!(!checker.can_delete());
        assert!(checker.check_permission(Operation::Read).is_ok());
        assert!(checker.check_permission(Operation::Write).is_ok());
        assert!(checker.check_permission(Operation::Delete).is_err());
    }

    #[test]
    fn test_permission_error_message() {
        let config = create_test_config(false, false, false);
        let checker = PermissionChecker::new(&config);

        let read_err = checker.check_permission(Operation::Read).unwrap_err();
        assert_eq!(
            read_err.to_string(),
            "Permission denied for read operation: read operations are disabled by configuration"
        );

        let write_err = checker.check_permission(Operation::Write).unwrap_err();
        assert_eq!(
            write_err.to_string(),
            "Permission denied for write operation: write operations are disabled by configuration"
        );

        let delete_err = checker.check_permission(Operation::Delete).unwrap_err();
        assert_eq!(
            delete_err.to_string(),
            "Permission denied for delete operation: delete operations are disabled by configuration"
        );
    }

    #[test]
    fn test_operation_display() {
        assert_eq!(Operation::Read.to_string(), "read");
        assert_eq!(Operation::Write.to_string(), "write");
        assert_eq!(Operation::Delete.to_string(), "delete");
    }
}
