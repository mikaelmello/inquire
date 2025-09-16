use quote::format_ident;

pub fn enum_name_to_module_name(enum_name: &str) -> proc_macro2::Ident {
    format_ident!("__inquire_enum_choice_for_{}", enum_name.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum_name_to_module_name() {
        let module_name = enum_name_to_module_name("TestEnum");
        assert_eq!(
            module_name.to_string(),
            "__inquire_enum_choice_for_testenum"
        );
    }

    #[test]
    fn test_enum_name_with_underscores() {
        let module_name = enum_name_to_module_name("Test_Enum_Name");
        assert_eq!(
            module_name.to_string(),
            "__inquire_enum_choice_for_test_enum_name"
        );
    }
}
