enum TestEnum {
    Common,
    Uncommon,
    Rare
}

impl TestEnum {
    fn run(&self) {
        match *self {
             TestEnum::Common => { println!("I am Common!"); },
             TestEnum::Uncommon => {println!("I am uncommon!"); }
             TestEnum::Rare => { println!("I am rare!!!!"); }
         }
    }
}

fn main() {
    let common = TestEnum::Common;
    common.run();
}