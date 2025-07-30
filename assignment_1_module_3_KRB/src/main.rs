
//Kevin Bueno assingmnment 1 module 3
// wanted to add some pseudocde to explain the code like on last module
use std::fs::File;
use std::io::{Write, BufReader, BufRead}; //file writing and reading

// Define the Book structure
struct Book {
    title: String,
    author: String,
    year: u16,
}

// Save books to file: one per line, fields separated by commas
fn save_books(books: &Vec<Book>, filename: &str) {
    let mut file = File::create(filename).expect("Failed to create file");

    for book in books {
        // Write each book as: title,author,year
        writeln!(file, "{},{},{}", book.title, book.author, book.year)
            .expect("Failed to write to file");
    }
}

// Load books from file into a Vec
fn load_books(filename: &str) -> Vec<Book> {
    let file = File::open(filename).expect("Failed to open file");
    let reader = BufReader::new(file);

    let mut books = Vec::new(); // Vector to store books

    for line_result in reader.lines() {
        let line = line_result.expect("Failed to read line");
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() == 3 {
            let title = parts[0].to_string();
            let author = parts[1].to_string();
            let year = parts[2].parse::<u16>().unwrap_or(0); 
            books.push(Book { title, author, year });
        }
    }

    books
}

fn main() {
    let books = vec![
        Book {
            title: "1984".to_string(),
            author: "George Orwell".to_string(),
            year: 1949,
        },
        Book {
            title: "To Kill a Mockingbird".to_string(),
            author: "Harper Lee".to_string(),
            year: 1960,
        },
    ];

    // Save to file
    save_books(&books, "books.txt");
    println!("Books saved to file.");

    // Load back from file
    let loaded_books = load_books("books.txt");
    println!("Loaded books:");
    for book in loaded_books {
        println!("{} by {}, published in {}", book.title, book.author, book.year);
    }
}
