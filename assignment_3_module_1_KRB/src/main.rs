fn check_guess(guess: i32, secret: i32) -> i32 {
    if guess == secret {
        0
    } else if guess > secret {
        1
    } else {
        -1
    }
}

fn main() {
    let secret_number = 18;
    let guesses = [10, 15, 20, 18]; // Simulated user guesses

    let mut attempts = 0;

    for &guess in guesses.iter() {
        attempts += 1;
        let result = check_guess(guess, secret_number);

        if result == 0 {
            println!("Guess {guess}: Correct!");
            break;
        } else if result == 1 {
            println!("Guess {guess}: Too high.");
        } else {
            println!("Guess {guess}: Too low.");
        }
    }

    println!("It took {attempts} guess(es) to find the number.");
}
