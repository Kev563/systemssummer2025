//Kevin Bueno Assignment 1

const FREEZING_POINT_F: f64 = 32.0;

fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - FREEZING_POINT_F) * 5.0 / 9.0
}

fn main() {
    let fahrenheit: f64 = 32.0;
    let celsius = fahrenheit_to_celsius(fahrenheit);
    println!("{fahrenheit}°F is {celsius:.2}°C");

    for i in 1..=5 {
        let next_f = fahrenheit + i as f64;
        let next_c = fahrenheit_to_celsius(next_f);
        println!("{next_f}°F is {next_c:.2}°C");
    }
}
