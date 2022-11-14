use snake::Board;

fn main() {
    let mut foo = 1usize;
    println!("{}", foo);

    foo -= 1;
    println!("{}", foo);

    let opt = foo.wrapping_sub(1);
    println!("{:#?}", opt);


    // let b = Board::create(10, 10);
    // println!("{}", b);
    // println!("{}", b.get((0,1)));
    // let hi: u8 = 0b0;
    // println!("{} {}", hi, !hi);
    // let hid: u8 = 0b1;
    // println!("{} {}", hid, !hid);
}
