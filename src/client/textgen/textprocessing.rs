/** Splits a vector of strings into a  */
fn vec_word_split<'a>(words: Vec<String>) -> (Vec<&'a str>, Vec<&'a str>) {
    let mut vec1: Vec<&str> = vec![];
    let mut vec2: Vec<&str> = vec![];
    let window = words.windows(2);
    todo!()
}
