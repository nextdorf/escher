use escher_schedule::*;

fn main() {
    //let numbers = vec![1,2,3,4];
    let numbers: Vec<usize> = vec![1];
    println!("Numbers: {:?}", numbers);

    let mut split_iter = SplitLastIter::from_iter(numbers.iter());

    println!("Last One: {:?}", split_iter.get_last());
    loop {
        if let Some(i) = split_iter.next() {
            println!("i: {}", i);
        } else {
            break;
        }
    }
    println!("Last One: {:?}", split_iter.get_last());

}
