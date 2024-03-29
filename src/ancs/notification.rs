use libnotify::Notification;

#[derive(Debug)]
pub struct ANCSNotification {
    notification: Notification,
    title: Option<String>,
    message: Option<String>,
    icon: Option<String>,
}

impl ANCSNotification {
    pub fn new() -> Self {
        let n = Notification::new(String::new().as_str(), None, None);

        Self {
            notification: n,
            title: None,
            message: None,
            icon: None,
        }
    }

    pub fn update(&mut self, title: Option<String>, message: Option<String>) {
        if title.is_some() {
            self.title = title;
        }
        if message.is_some() {
            self.message = message;
        }
        self.notification.update(
            self.title.as_deref().unwrap_or(String::new().as_str()),
            self.message.as_deref(),
            self.icon.as_deref(),
        ).unwrap();
        if self.displayable() {
            self.show();
        }
    }

    pub fn show(&self) {
        self.notification.show().unwrap();
    }

    pub fn displayable(&self) -> bool {
        self.title.is_some() && self.message.is_some()
    }
}
