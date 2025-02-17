fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    let name = "Rustacean";
    let greeting = greet(name);
    println!("{}", greeting);

    // Demonstrate some basic Rust features
    let numbers = vec![1, 2, 3, 4, 5];
    
    // Using iterator and closure
    let sum: i32 = numbers
        .iter()
        .filter(|&x| x % 2 == 0)  // Get even numbers
        .sum();
    
    println!("Sum of even numbers: {}", sum);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(greet("World"), "Hello, World!");
    }
}
