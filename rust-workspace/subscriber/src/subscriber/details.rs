pub struct NonEmptyString(String);

pub struct SubscriberName(NonEmptyString);

pub struct SubscriberEmail(NonEmptyString);

pub struct SubscriberExternalId(NonEmptyString);

pub struct SubscriberDetails {
    pub name: SubscriberName,
    pub email: SubscriberEmail,
    pub external_id: SubscriberExternalId,
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0.as_ref()
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0.as_ref()
    }
}

impl AsRef<str> for SubscriberExternalId {
    fn as_ref(&self) -> &str {
        &self.0.as_ref()
    }
}

impl AsRef<str> for NonEmptyString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for NonEmptyString {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err("value cannot be empty".to_string());
        }
        Ok(NonEmptyString(value))
    }
}

impl TryFrom<String> for SubscriberName {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let non_empty_string = NonEmptyString::try_from(value)?;
        Ok(SubscriberName(non_empty_string))
    }
}

impl TryFrom<String> for SubscriberExternalId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let non_empty_string = NonEmptyString::try_from(value)?;
        Ok(SubscriberExternalId(non_empty_string))
    }
}

impl TryFrom<String> for SubscriberEmail {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        use validator::validate_email;
        let non_empty_string = NonEmptyString::try_from(value)?;

        let is_valid = validate_email(non_empty_string.as_ref());
        if is_valid {
            return Ok(SubscriberEmail(non_empty_string));
        }
        Err(format!("{} is an invalid email", non_empty_string.as_ref()))
    }
}
