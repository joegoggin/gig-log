//! CSS class string composition helpers.

/// Provides helper methods for composing CSS class name strings.
pub struct ClassNameUtil {
    main_class: String,
    class_prop: Option<String>,
    parent_class: Option<String>,
    is_layout: bool,
}

impl ClassNameUtil {
    pub fn new(main_class: impl Into<String>, class_prop: Option<String>) -> Self {
        Self {
            main_class: main_class.into(),
            class_prop,
            parent_class: None,
            is_layout: false,
        }
    }

    pub fn new_layout_class_name(
        main_class: impl Into<String>,
        class_prop: Option<String>,
    ) -> Self {
        Self {
            main_class: main_class.into(),
            class_prop,
            parent_class: None,
            is_layout: true,
        }
    }

    pub fn new_with_parent(
        parent_class: impl Into<String>,
        main_class: impl Into<String>,
        class_prop: Option<String>,
    ) -> Self {
        Self {
            main_class: main_class.into(),
            class_prop,
            parent_class: Some(parent_class.into()),
            is_layout: false,
        }
    }

    pub fn get_root_class(&self) -> String {
        match &self.class_prop {
            Some(class_prop) if !self.is_layout => format!("{} {}", self.main_class, class_prop),
            _ => self.main_class.to_string(),
        }
    }

    pub fn get_root_class_with_parent(&self) -> String {
        match &self.parent_class {
            Some(parent_class) if !self.is_layout => {
                let root_class = self.get_root_class();

                format!("{} {}", parent_class, root_class)
            }
            _ => self.get_root_class(),
        }
    }

    pub fn get_root_class_with_parent_variation(&self, variation: &str) -> String {
        match &self.parent_class {
            Some(parent_class) => {
                let root_class = self.get_root_class();
                let parent_variation = format!("{}--{}", parent_class, variation);

                format!("{} {} {}", parent_class, parent_variation, root_class)
            }
            None => self.get_root_class(),
        }
    }

    pub fn get_sub_class(&self, sub_class: &str) -> String {
        format!("{}__{}", self.main_class, sub_class)
    }

    pub fn get_root_variation(&self, variation: &str) -> String {
        let root_class = self.get_root_class();
        let variation_class = format!("{}--{}", self.main_class, variation);

        format!("{} {}", root_class, variation_class)
    }

    pub fn get_sub_class_variation(&self, sub_class: &str, variation: &str) -> String {
        let sub_class = self.get_sub_class(sub_class);

        format!("{} {}--{}", sub_class, sub_class, variation)
    }

    pub fn get_content_class(&self) -> String {
        let content_class = format!("{}__content", self.main_class);

        if let Some(class_prop) = &self.class_prop
            && self.is_layout
        {
            return format!("{} {}", content_class, class_prop);
        }

        content_class
    }
}
