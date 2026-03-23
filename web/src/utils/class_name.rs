pub struct ClassNameUtil;

impl ClassNameUtil {
    pub fn add_optional_class(
        main_class: impl Into<String>,
        optional_class: Option<String>,
    ) -> String {
        match optional_class {
            Some(class) => format!("{} {}", main_class.into(), class),
            None => main_class.into(),
        }
    }
}
