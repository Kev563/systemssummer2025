// Kevin Bueno Assignment 2


fn is_even(n: i32) -> bool {
    n % 2 == 0
}

fn main() {
    let numbers = [12, 7, 15, 22, 9, 5, 18, 30, 4, 1];

    println!("--- Number Analysis ---");
    for &num in numbers.iter() {
        if num % 3 == 0 && num % 5 == 0 {
            println!("{num}: FizzBuzz");
        } else if num % 3 == 0 {
            println!("{num}: Fizz");
        } else if num % 5 == 0 {
            println!("{num}: Buzz");
        } else if is_even(num) {
            println!("{num}: Even");
        } else {
            println!("{num}: Odd");
        }
    }

    // While loop to sum all numbers
    let mut index = 0;
    let mut sum = 0;
    while index < numbers.len() {
        sum += numbers[index];
        index += 1;
    }
    println!("\nSum of all numbers: {sum}");

    // Loop to find largest number
    let mut max = numbers[0];
    let mut i = 1;
    loop {
        if i >= numbers.len() {
            break;
        }
        if numbers[i] > max {
            max = numbers[i];
        }
        i += 1;
    }
    println!("Largest number: {max}");
}
