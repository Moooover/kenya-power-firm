#[macro_export]
macro_rules! string_key {
    ($TypeName: ident) => {
        #[derive(
            Clone,
            Debug,
            Default,
            Eq,
            Hash,
            Ord,
            PartialEq,
            PartialOrd,
            serde::Serialize,
            serde::Deserialize,
        )]
        pub struct $TypeName(String);

        impl $TypeName {
            pub fn inner(&self) -> String {
                self.0.clone()
            }

            pub fn new(value: String) -> Self {
                $TypeName(value)
            }
        }

        impl PartialEq<str> for $TypeName {
            fn eq(&self, other: &str) -> bool {
                &self.inner() == other
            }
        }

        impl std::fmt::Display for $TypeName {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<String> for $TypeName {
            fn from(id: String) -> Self {
                $TypeName(id)
            }
        }

        impl From<&str> for $TypeName {
            fn from(id: &str) -> Self {
                $TypeName(id.to_owned())
            }
        }

        impl From<$TypeName> for String {
            fn from(id: $TypeName) -> Self {
                id.inner()
            }
        }

        impl AsRef<str> for $TypeName {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}
