/// SensitiveString is a New Type that prevents sensitive strings like API keys, passwords and any
/// other sensitive information of being displayed unintentionally in any output. It works by wrapping
/// the original value and overriding Display and Debug traits.
///
/// Access to the original value can be obtained by derefing the SensitiveString to a String reference.
pub(crate) mod sensitive_string {
    use serde::{Deserialize, Serialize};

    use std::convert::Infallible;
    use std::fmt::{Debug, Display, Formatter};
    use std::ops::Deref;
    use std::str::FromStr;

    const MASK: &str = "******";

    #[derive(Clone, Serialize, Deserialize)]
    pub struct SensitiveString(String);

    impl Display for SensitiveString {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", MASK)
        }
    }

    impl Debug for SensitiveString {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_tuple("SensitiveString").field(&MASK).finish()
        }
    }

    impl FromStr for SensitiveString {
        type Err = Infallible;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(SensitiveString(s.into()))
        }
    }

    impl Deref for SensitiveString {
        type Target = String;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_sensitive_string() {
            let secret = "123456";
            let sensitive = SensitiveString(secret.into());

            // test it hides the content when displaying and debugging
            assert_eq!(sensitive.to_string(), MASK);
            assert_eq!(format!("{}", sensitive), MASK);
            assert_eq!(
                format!("{:?}", sensitive),
                format!("SensitiveString(\"{}\")", MASK)
            );

            // test it allows access to the secret string when requested
            let sensitive_deref: &str = &sensitive;
            assert_eq!(sensitive_deref, secret);
        }
    }
}
