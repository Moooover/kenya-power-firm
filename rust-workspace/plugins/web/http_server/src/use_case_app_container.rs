use use_cases::{App, AppImpl};

pub struct UseCaseAppContainer(Box<dyn App>);

impl UseCaseAppContainer {
    pub fn new(app: AppImpl) -> Self {
        Self(Box::new(app))
    }

    pub fn get_client(&self) -> &dyn App {
        self.0.as_ref()
    }
}
