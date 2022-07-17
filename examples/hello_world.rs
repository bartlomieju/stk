use stokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new();
    rt.spawn(async {
        println!("hello world");
    });

    rt.run();
}
