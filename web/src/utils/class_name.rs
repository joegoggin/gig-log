pub struct ClassNameUtil;

impl ClassNameUtil {
    pub fn add_class(current_class: impl Into<String>, new_class: impl Into<String>) -> String {
        format!("{} {}", current_class.into(), new_class.into())
    }

    pub fn add_optional_class(
        current_class: impl Into<String>,
        optional_class: Option<&str>,
    ) -> String {
        match optional_class {
            Some(class) => format!("{} {}", current_class.into(), class),
            None => current_class.into(),
        }
    }
}
