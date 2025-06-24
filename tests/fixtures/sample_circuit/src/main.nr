fn main(a: Field, b: Field) -> pub Field {
    let sum = a + b;
    assert(sum != 0); // Simple constraint to make it more than trivial
    sum
}
