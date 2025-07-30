//fn main() {
  //  println!("Hello, Kevin Bueno!");
// }

fn main() {
    // Shadowing
    let x = 5;
    let x = x + 1;  // Creates a new variable
    
    // Mutation
    let mut y = 5;
    y = y + 1;  // Modifies the existing variable
    
    println!("x: {}, y: {}", x, y);
}