extern crate notify_rust;


#[cfg(not(target_os = "windows"))]
pub fn notify(msg_title: &str, msg_body: &str) {
    notify_rust::Notification::new()
        .summary(msg_title)
        .body(msg_body)
        .show()
        .unwrap();
}

#[cfg(target_os = "windows")]
fn notify(msg_title: &str, msg_body: &str) {
    extern crate winrt;
    use winrt::*;
    use winrt::windows::data::xml::dom::*;
    use winrt::windows::ui::notifications::*;
    unsafe {
        let mut toast_xml =
            ToastNotificationManager::get_template_content(ToastTemplateType_ToastText02).unwrap();
        let mut toast_text_elements = toast_xml
            .get_elements_by_tag_name(&FastHString::new("text"))
            .unwrap();

        toast_text_elements
            .item(0)
            .unwrap()
            .append_child(&*toast_xml
                .create_text_node(&FastHString::from(msg_title))
                .unwrap()
                .query_interface::<IXmlNode>()
                .unwrap())
            .unwrap();
        toast_text_elements
            .item(1)
            .unwrap()
            .append_child(&*toast_xml
                .create_text_node(&FastHString::from(msg_body))
                .unwrap()
                .query_interface::<IXmlNode>()
                .unwrap())
            .unwrap();

        let toast = ToastNotification::create_toast_notification(&*toast_xml).unwrap();
        ToastNotificationManager::create_toast_notifier_with_id(&FastHString::new("aria2c-gui"))
            .unwrap()
            .show(&*toast)
            .unwrap();
    }
}
