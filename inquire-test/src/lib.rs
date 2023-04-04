//
// TODO: figure out a way to test it...
//

// #[cfg(test)]
// mod tests {
//     use inquire::length;
//     use inquire::CustomUserError;
//     use inquire_derive::InquireForm;

//     fn suggester(val: &str) -> Result<Vec<String>, CustomUserError> {
//         let suggestions = [
//             "Andrew",
//             "Charles",
//             "Christopher",
//             "Daniel",
//             "David",
//             "Donald",
//             "Edward",
//             "George",
//             "James",
//             "John",
//             "Johnny",
//             "Kevin",
//             "Mark",
//             "Michael",
//             "Paul",
//             "Richard",
//             "Robert",
//             "Steven",
//             "Thomas",
//             "William",
//         ];

//         let val_lower = val.to_lowercase();

//         Ok(suggestions
//             .iter()
//             .filter(|s| s.to_lowercase().contains(&val_lower))
//             .map(|s| String::from(*s))
//             .collect())
//     }

//     #[test]
//     fn basic_inquire() {
//         #[derive(Debug, InquireForm)]
//         pub struct TestStruct {
//             #[inquire(text(prompt_message = "What's your main_text?"))]
//             pub main_text: String,
//             #[inquire(text())]
//             pub text: String,
//             #[inquire(text(
//                 prompt_message = "What's your path?",
//                 initial_value = "/my/path",
//                 default_value = "/my/path",
//                 placeholder_value = "/my/path",
//                 help_message = "insert my path",
//                 page_size = 1,
//                 // TODO: Try to figure out another way
//                 validators = "vec![Box::new(length!(5))]",
//                 // TODO: Is it the right path?
//                 autocompleter = "&suggester"
//             ))]
//             pub path: String,
//         }

//         impl Default for TestStruct {
//             fn default() -> Self {
//                 Self {
//                     main_text: String::from("test"),
//                     text: String::from("test"),
//                     path: String::new(),
//                 }
//             }
//         }

//         let mut ex = TestStruct {
//             main_text: String::from("difference1"),
//             text: String::from("difference2"),
//             path: String::new(),
//         };
//         ex.inquire_mut().unwrap();
//         println!("{:?}", ex);
//         let df = TestStruct::from_inquire();
//         println!("{:?}", df)
//     }

//     #[test]
//     fn select_inquire() {
//         fn get_list_options() -> Vec<String> {
//             vec!["rings".into(), "power".into()]
//         }

//         #[derive(Debug, InquireForm)]
//         pub struct TestStruct {
//             #[inquire(select(
//                 prompt_message = "What's your selection?",
//                 options = "get_list_options()"
//             ))]
//             pub path: String,
//         }

//         impl Default for TestStruct {
//             fn default() -> Self {
//                 Self {
//                     path: String::from("test"),
//                 }
//             }
//         }

//         let mut ex = TestStruct {
//             path: String::new(),
//         };
//         ex.inquire_mut().unwrap();
//         println!("{:?}", ex);
//     }
// }
