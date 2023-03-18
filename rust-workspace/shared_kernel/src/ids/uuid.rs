#[macro_export]
macro_rules! uuid_key {
    ($TypeName: ident) => {
        #[derive(
            Clone,
            Copy,
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
        pub struct $TypeName(uuid::Uuid);

        impl $TypeName {
            pub fn new() -> Self {
                $TypeName(uuid::Uuid::new_v4())
            }

            pub fn inner(&self) -> uuid::Uuid {
                self.0
            }
        }

        impl PartialEq<uuid::Uuid> for $TypeName {
            fn eq(&self, other: &uuid::Uuid) -> bool {
                self.inner() == *other
            }
        }

        impl std::fmt::Display for $TypeName {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<uuid::Uuid> for $TypeName {
            fn from(id: uuid::Uuid) -> Self {
                $TypeName(id)
            }
        }

        impl From<$TypeName> for uuid::Uuid {
            fn from(id: $TypeName) -> Self {
                id.inner()
            }
        }
    };
}
