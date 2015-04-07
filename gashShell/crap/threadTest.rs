use std::thread;

fn main() {
    let handle = thread::scoped(get_closure(5));
}


fn get_closure(x: i32) -> || {
    move || {println!("The number I found is {}", x);}
}