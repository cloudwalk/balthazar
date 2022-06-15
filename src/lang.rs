/// SensitiveString is a New Type that prevents sensitive strings like API keys, passwords and any
/// other sensitive information of being displayed unintentionally in any output. It works by wrapping
/// the original value and overriding Display and Debug traits.
///
/// Access to the original value can be obtained by derefing the SensitiveString to a String reference.
pub mod sensitive {
    use serde::{Deserialize, Serialize};

    use std::fmt::{Debug, Display, Formatter};
    use std::ops::Deref;
    use std::str::FromStr;

    const MASK: &str = "******";

    #[derive(Clone, Serialize, Deserialize)]
    pub struct Sensitive<T>(T);

    impl<T> Sensitive<T> {
        pub fn new(value: T) -> Self {
            Self(value)
        }
    }

    impl<T> Display for Sensitive<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", MASK)
        }
    }

    impl<T> Debug for Sensitive<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_tuple("Sensitive<T>").field(&MASK).finish()
        }
    }

    impl<T> From<T> for Sensitive<T> {
        fn from(t: T) -> Self {
            Self(t)
        }
    }

    impl<T> FromStr for Sensitive<T>
    where
        T: FromStr,
    {
        type Err = T::Err;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match T::from_str(s) {
                Ok(t) => Ok(Sensitive(t)),
                Err(e) => Err(e),
            }
        }
    }

    impl<T> Deref for Sensitive<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[deprecated(note = "please use Sensitive<String> instead")]
    pub type SensitiveString = Sensitive<String>;

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_sensitive_string() {
            let secret = "123456";
            let sensitive = Sensitive::<String>(secret.to_string());

            // test it hides the content when displaying and debugging
            assert_eq!(sensitive.to_string(), MASK);
            assert_eq!(format!("{}", sensitive), MASK);
            assert_eq!(
                format!("{:?}", sensitive),
                format!("Sensitive<T>(\"{}\")", MASK)
            );

            // test it allows access to the secret string when requested
            let sensitive_deref: &str = &sensitive;
            assert_eq!(sensitive_deref, secret);
        }

        #[test]
        fn test_sensitive_into() {
            let s: Sensitive<String> = "123456".to_string().into();
            assert_eq!(s.deref(), "123456");

            let v: Sensitive<Vec<u8>> = vec![1, 2, 3].into();
            assert_eq!(*v.deref(), vec![1, 2, 3]);
        }
    }
}
