// Kevin Bueno assignment 2 module 2
fn most_frequent_word(text: &str) -> (String, usize) {
    let words: Vec<&str> = text.split_whitespace().collect(); // Split text by spaces
    let mut max_word = String::new(); // Will store the word with the highest count
    let mut max_count = 0; // Will store how many times it appears

    // Outer loop goes through each word
    for i in 0..words.len() {
        let mut count = 1; // Start counting this word
        for j in (i + 1)..words.len() {
            if words[i] == words[j] {
                count += 1; // Count another occurrence
            }
        }

        // Update max if found a more frequent word
        if count > max_count {
            max_word = words[i].to_string(); // Copy the word
            max_count = count;
        }
    }

    (max_word, max_count) // Return 
}

fn main() {
    let text = "the quick brown fox jumps over the lazy dog the quick brown fox";
    let (word, count) = most_frequent_word(text); // Borrow text as input
    println!("Most frequent word: \"{}\" ({} times)", word, count);
}
