// Kevin Bueno Assingnment 1 module 2
fn sum_with_step(total: &mut i32, low: i32, high: i32, step: i32) {
    let mut i = low; // Start from the low value
    while i <= high { // Continue while i is within the range
        *total += i;  // Add current i value to the total 
        i += step;    // Move to the next number using the step
    }
}

fn main() {
    let mut result = 0; // This will store sum

    // function to sum from 0 to 100 with step 1
    sum_with_step(&mut result, 0, 100, 1);
    println!("Sum 0 to 100, step 1: {}", result);

    // Reset result and sum from 0 to 10 
    result = 0;
    sum_with_step(&mut result, 0, 10, 2);
    println!("Sum 0 to 10, step 2: {}", result);

    // Reset result and sum from 5 to 15
    result = 0;
    sum_with_step(&mut result, 5, 15, 3);
    println!("Sum 5 to 15, step 3: {}", result);
}
