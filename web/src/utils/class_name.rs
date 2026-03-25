pub struct ClassNameUtil {
    main_class: String,
    class_prop: Option<String>,
}

impl ClassNameUtil {
    pub fn new(main_class: impl Into<String>, class_prop: Option<String>) -> Self {
        Self {
            main_class: main_class.into(),
            class_prop,
        }
    }

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

    pub fn get_root_class(&self) -> String {
        match &self.class_prop {
            Some(class_prop) => format!("{} {}", self.main_class, class_prop),
            None => self.main_class.to_string(),
        }
    }

    pub fn get_sub_class(&self, sub_class: &str) -> String {
        format!("{}__{}", self.main_class, sub_class)
    }

    pub fn get_root_variation(&self, variation: &str) -> String {
        let root_class = self.get_root_class();

        format!("{} {}--{}", root_class, root_class, variation)
    }

    pub fn get_sub_class_variation(&self, sub_class: &str, variation: &str) -> String {
        let sub_class = self.get_sub_class(sub_class);

        format!("{} {}--{}", sub_class, sub_class, variation)
    }
}
