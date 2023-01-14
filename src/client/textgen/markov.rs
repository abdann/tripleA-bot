use lazy_static::lazy_static;
use regex::Captures;
use regex::Regex;

/** Extracts complete sentences from a string into a vector. Expects all newlines and tab characters to be replaced with spaces*/
fn extract_sentences(string_buffer: &str) -> Vec<&str> {
    // expects \n and \t to be replaced with " " in the string_buffer
    lazy_static! {
        static ref SENTENCE_EXTRACT: Regex =
            Regex::new("[A-Za-z0-9'\";:,]([A-Za-z0-9\\s'\";:,]|\\.[^\\s])*[.!?]")
                .expect("Should be valid regex");
    } // Make this lazy static since this is an expensive operation and we only need it compiled once.
    SENTENCE_EXTRACT
        .find_iter(string_buffer)
        .map(|thing| thing.as_str())
        .collect()
}

/// Filters a string so it can be inserted into the Markov data set.
///
/// Removes links, User IDs, emotes, animated emotes, non alphanumeric characters, line feeds, extra whitespace, and role IDs.
///
/// Replaces uppercase letters with their lowercase variants.
pub fn filter_string_pre_processing(msg: &str) -> String {
    lazy_static! {static ref LINK_EXTRACT: Regex = Regex::new(r#"(?:(?:https?|ftp)://|\b(?:[a-z\d]+\.))(?:(?:[^\s()<>]+|\((?:[^\s()<>]+|(?:\([^\s()<>]+\)))?\))+(?:\((?:[^\s()<>]+|(?:\(?:[^\s()<>]+\)))?\)|[^\s`!()\[\]{};:'".,<>?«»“”‘’]))?"#)
    .expect("Invalid regular expression");}

    let mut str = LINK_EXTRACT.replace_all(msg, "").into_owned();
    while str.ends_with(' ') {
        str.pop();
    }

    let mut filtered_message = str;
    static PATTERNS: once_cell::sync::OnceCell<Vec<Regex>> = once_cell::sync::OnceCell::new();
    PATTERNS
        .set(vec![
            Regex::new(r"<:?(\w+:)(\d+)>").expect("Invalid regular expression"), //emote regex
            Regex::new(r"<a:?(\w+:)(\d+)>").expect("Invalid regular expression"), //animated emote regex
            Regex::new(r#"[,.!"\#$()=?*<>{}\[\]\\\|Łł@*;:+~ˇ^˘°˛`´˝]"#)
                .expect("Invalid regular expression"), //non alphanumeric regex
            Regex::new(r"^(\d{18})$").expect("Invalid regular expression"), //remaining numbers from users regex
            Regex::new(r"\n").expect("Invalid regular expression"),         //line feed regex
            Regex::new(r"[ ]{3}|[ ]{2}").expect("Invalid regular expression"), //double and triple whitespace regex
            Regex::new(r"<@&(\d+)>").expect("Invalid regular expression"),     // role regex
        ])
        .unwrap();

    let upper_case_regex = Regex::new(r"[A-Z][a-z0-9_-]{1,}").expect("Invalid regular expression");

    loop {
        let mut number_of_matches: u16 = 0;

        for regex in PATTERNS.get().unwrap() {
            while regex.is_match(&filtered_message) {
                number_of_matches += 1;
                filtered_message = regex.replace_all(&filtered_message, " ").into_owned();
            }
        }
        while upper_case_regex.is_match(&filtered_message) {
            number_of_matches += 1;
            filtered_message = upper_case_regex
                .replace(&filtered_message, |caps: &Captures| caps[0].to_lowercase())
                .into_owned();
        }
        if number_of_matches == 0 {
            break;
        }
    }

    return filtered_message.trim().to_owned();
}
// /**Formats all files in the corpi/ directory into the proper form for a markov chain reader. In the bot, this would be called with a value of true*/
// fn format_corpus_files(delete_old_files: bool) {
//     // get the names of all the text files in the corpus directory.
//     let valid_texts = get_all_valid_texts();
//     for file_name in valid_texts {
//         // Open each file
//         let mut base_file = File::open(&file_name)
//             .expect("We should be able to read these files since we *just* checked them.");

//         // Make a string buffer
//         let mut temp_string = String::new();
//         // Read in base file into temp_string
//         base_file
//             .read_to_string(&mut temp_string)
//             .expect("Should be able to write to string buffer.");

//         temp_string = temp_string.replace("\n", " ").replace("\t", " "); // Replaces newlines and tabs with empty strings.

//         // extract sentences, then return as a sentence per newline.
//         temp_string = extract_sentences(&temp_string).into_iter().join("\n");

//         // choose whether to save as new files or rename with a _temp suffix
//         let final_name: String;
//         if delete_old_files {
//             final_name = file_name;
//         } else {
//             final_name = file_name.replace(".txt", "_temp.txt");
//         }
//         //Create the final file naming it final name
//         let mut final_file =
//             File::create(final_name).expect("Should be able to create a write only file.");
//         // Write the temp_string to the final_file
//         final_file
//             .write(temp_string.as_bytes())
//             .expect("Should be able to write to created file.");
//     }
// }
