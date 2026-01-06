use sample_project::{add, subtract, Calculator};

fn main() {
    // Test basic math operations
    let x = 5;
    let y = 10;
    let result = add(x, y);
    println!("Result: {}", result);

    let diff = subtract(y, x);
    println!("Difference: {}", diff);

    // Test calculator trait
    use_calculator();
}

fn use_calculator() {
    let calc = sample_project::Adder;
    let sum = calc.calculate(15, 25);
    println!("Calculator result: {}", sum);
}
