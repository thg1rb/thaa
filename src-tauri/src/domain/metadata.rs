//! Field-level availability for metadata that operating systems may restrict.

/// Why a field could not be observed for a particular process.
///
/// Whole-query failures and process disappearance are not field availability
/// states; those belong to future provider/application error contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnavailableReason {
    PermissionDenied,
    Unsupported,
    Inaccessible,
    ProviderLimitation,
}

/// A metadata value that was observed, or an explicit reason it is unavailable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldAvailability<T> {
    Available(T),
    Unavailable(UnavailableReason),
}

#[cfg(test)]
mod tests {
    use super::{FieldAvailability, UnavailableReason};
    use std::ffi::OsString;

    #[test]
    fn available_values_preserve_empty_and_hostile_text_as_data() {
        let empty = FieldAvailability::Available(OsString::new());
        let unusual = FieldAvailability::Available(OsString::from(
            "Sample Process\n\u{1b}[31mnot interpreted\u{1b}[0m",
        ));

        assert_eq!(empty, FieldAvailability::Available(OsString::new()));
        assert_eq!(
            unusual,
            FieldAvailability::Available(OsString::from(
                "Sample Process\n\u{1b}[31mnot interpreted\u{1b}[0m"
            ))
        );
    }

    #[test]
    fn unavailable_reasons_remain_distinct() {
        assert_ne!(
            FieldAvailability::<OsString>::Unavailable(UnavailableReason::PermissionDenied),
            FieldAvailability::Unavailable(UnavailableReason::Unsupported)
        );
        assert_ne!(
            FieldAvailability::<OsString>::Unavailable(UnavailableReason::Inaccessible),
            FieldAvailability::Unavailable(UnavailableReason::ProviderLimitation)
        );
    }
}
