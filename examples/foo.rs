fn main() {
    let foo: isize = 5;
    let bar: usize = foo.try_into().expect("couldn't fit it brah");
    println!("{}", bar);
}