use std::{borrow::Borrow, collections::HashMap};

/** Splits a vector of objects */
pub fn vec_word_split(words: Vec<i32>) -> Vec<Vec<i32>> {
    let mut returnable: Vec<Vec<i32>> = vec![];
    // let words = words
    //     .into_iter()
    //     .map(|word| word.as_ref())
    //     .collect::<Vec<&str>>();
    let windows = words.windows(2);
    let mut inc: u32 = 0;
    for window in windows.into_iter() {
        returnable.push(window.to_vec());
        inc += 1;
    }
    returnable
}

#[cfg(test)]
mod tests {

    use super::*;

    // #[test]
    // fn test_split_strings_even_words() {
    //     let test_input = vec![
    //         "hi".to_owned(),
    //         "my".to_owned(),
    //         "name".to_owned(),
    //         "minecraft".to_owned(),
    //     ];
    //     let mut correct_map = HashMap::new();
    //     correct_map.insert(0u32, vec!["hi".to_owned(), "my".to_owned()]);
    //     correct_map.insert(1u32, vec!["my".to_owned(), "name".to_owned()]);
    //     correct_map.insert(2u32, vec!["name".to_owned(), "minecraft".to_owned()]);
    //     assert_eq!(correct_map, vec_word_split(test_input));
    // }
    // #[test]
    // fn test_split_strings_odd_words() {
    //     let test_input = vec!["hi".to_owned(), "my".to_owned(), "name".to_owned()];
    //     let mut correct_map = HashMap::new();
    //     correct_map.insert(0u32, vec!["hi".to_owned(), "my".to_owned()]);
    //     correct_map.insert(1u32, vec!["my".to_owned(), "name".to_owned()]);
    //     assert_eq!(correct_map, vec_word_split(test_input));
    // }
    // #[test]
    // fn test_split_strings_odd_words() {
    //     let test_input = vec!["hi", "my", "name"];
    //     let mut correct_map = HashMap::new();
    //     correct_map.insert(0u32, vec!["hi", "my"]);
    //     correct_map.insert(1u32, vec!["my", "name"]);
    //     assert_eq!(correct_map, vec_word_split(test_input));
    // }
}
