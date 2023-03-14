use shared_kernel::non_empty_string;

non_empty_string!(SubscriberName);
non_empty_string!(SubscriberEmailInner);
non_empty_string!(SubscriberExternalId);

pub struct SubscriberEmail(SubscriberEmailInner);

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

pub struct SubscriberDetails {
    pub name: SubscriberName,
    pub email: SubscriberEmail,
    pub external_id: SubscriberExternalId,
}

impl TryFrom<String> for SubscriberEmail {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        use validator::validate_email;
        let non_empty_string = SubscriberEmailInner::try_from(value)?;

        let is_valid = validate_email(non_empty_string.as_ref());
        if is_valid {
            return Ok(SubscriberEmail(non_empty_string));
        }
        Err(format!("{} is an invalid email", non_empty_string.as_ref()))
    }
}
