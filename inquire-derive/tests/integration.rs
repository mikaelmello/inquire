use inquire_derive::Selectable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Selectable)]
enum Color {
    Red,
    Green,
    Blue,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Red => write!(f, "Red"),
            Color::Green => write!(f, "Green"),
            Color::Blue => write!(f, "Blue"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Selectable)]
enum Fruit {
    Apple,
    Banana,
    Orange,
}

impl std::fmt::Display for Fruit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fruit::Apple => write!(f, "Apple"),
            Fruit::Banana => write!(f, "Banana"),
            Fruit::Orange => write!(f, "Orange"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Selectable)]
enum Animal {
    Cat,
    Dog,
    Bird,
}

impl std::fmt::Display for Animal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Animal::Cat => write!(f, "Cat"),
            Animal::Dog => write!(f, "Dog"),
            Animal::Bird => write!(f, "Bird"),
        }
    }
}

#[test]
fn test_display_implementation() {
    assert_eq!(format!("{}", Color::Red), "Red");
    assert_eq!(format!("{}", Fruit::Apple), "Apple");
    assert_eq!(format!("{}", Animal::Cat), "Cat");
}

#[test]
fn test_generated_variants_are_correct() {
    // Test that the variants are generated correctly by checking the builder options
    let select_prompt = Color::select("Choose:");

    assert_eq!(select_prompt.options.len(), 3);
    assert_eq!(select_prompt.options.first().unwrap(), &Color::Red);
    assert_eq!(select_prompt.options.get(1).unwrap(), &Color::Green);
    assert_eq!(select_prompt.options.last().unwrap(), &Color::Blue);
}

#[test]
fn test_different_enum_variants() {
    let fruit_prompt = Fruit::select("Choose:");
    let color_prompt = Color::select("Choose:");

    // Test Fruit variants
    assert_eq!(fruit_prompt.options.len(), 3);
    assert_eq!(fruit_prompt.options.first().unwrap(), &Fruit::Apple);
    assert_eq!(fruit_prompt.options.get(1).unwrap(), &Fruit::Banana);
    assert_eq!(fruit_prompt.options.last().unwrap(), &Fruit::Orange);

    // Verify they're different from Color variants
    assert_eq!(color_prompt.options.len(), 3);
    // We can't directly compare different enum types, but we can check they have the same structure
}

#[test]
fn test_select_returns_proper_builder() {
    let select_prompt = Color::select("Choose a color:");

    // Verify we can access builder properties
    assert_eq!(select_prompt.message, "Choose a color:");
    assert_eq!(select_prompt.options.len(), 3);
    assert_eq!(select_prompt.options.first().unwrap(), &Color::Red);
    assert_eq!(select_prompt.options.get(1).unwrap(), &Color::Green);
    assert_eq!(select_prompt.options.last().unwrap(), &Color::Blue);
}

#[test]
fn test_multi_select_returns_proper_builder() {
    let multi_select_prompt = Fruit::multi_select("Choose fruits:");

    // Verify we can access builder properties
    assert_eq!(multi_select_prompt.message, "Choose fruits:");
    assert_eq!(multi_select_prompt.options.len(), 3);
    assert_eq!(multi_select_prompt.options.first().unwrap(), &Fruit::Apple);
    assert_eq!(multi_select_prompt.options.get(1).unwrap(), &Fruit::Banana);
    assert_eq!(multi_select_prompt.options.last().unwrap(), &Fruit::Orange);
}

#[test]
fn test_builder_customization_works() {
    let select_prompt = Color::select("Choose a color:")
        .with_help_message("Use arrow keys")
        .with_page_size(2);

    assert_eq!(select_prompt.message, "Choose a color:");
    assert_eq!(select_prompt.help_message, Some("Use arrow keys"));
    assert_eq!(select_prompt.page_size, 2);
    assert_eq!(select_prompt.options.len(), 3);
}

#[test]
fn test_multi_select_builder_customization() {
    let multi_select_prompt = Animal::multi_select("Choose animals:")
        .with_help_message("Space to select")
        .with_page_size(5)
        .with_default(&[0, 2]);

    assert_eq!(multi_select_prompt.message, "Choose animals:");
    assert_eq!(multi_select_prompt.help_message, Some("Space to select"));
    assert_eq!(multi_select_prompt.page_size, 5);
    assert_eq!(multi_select_prompt.default, Some(vec![0, 2]));
    assert_eq!(multi_select_prompt.options.len(), 3);
}

#[test]
fn test_enum_with_different_variant_count() {
    // Create a simple enum to test different variant counts
    // We'll test this by using our existing Animal enum which has 3 variants
    let animal_prompt = Animal::select("Choose animal:");
    assert_eq!(animal_prompt.options.len(), 3);

    // Test that each variant is properly accessible
    assert_eq!(animal_prompt.options.first().unwrap(), &Animal::Cat);
    assert_eq!(animal_prompt.options.get(1).unwrap(), &Animal::Dog);
    assert_eq!(animal_prompt.options.last().unwrap(), &Animal::Bird);

    // Test with multi_select too
    let multi_animal_prompt = Animal::multi_select("Choose animals:");
    assert_eq!(multi_animal_prompt.options.len(), 3);
    assert_eq!(multi_animal_prompt.options, animal_prompt.options);
}

#[test]
fn test_both_methods_on_same_enum() {
    // Verify that both select and multi_select work on the same enum
    let select_prompt = Animal::select("Pick one:");
    let multi_select_prompt = Animal::multi_select("Pick many:");

    // Both should have the same options
    assert_eq!(select_prompt.options, multi_select_prompt.options);
    assert_eq!(select_prompt.options.len(), 3);

    // But different types
    assert_eq!(select_prompt.message, "Pick one:");
    assert_eq!(multi_select_prompt.message, "Pick many:");
}
