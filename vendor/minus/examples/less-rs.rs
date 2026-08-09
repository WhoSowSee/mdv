use std::env::args;
use std::fs::File;
use std::io::{BufReader, Read};
use std::thread;

fn read_file(name: String, pager: minus::Pager) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(name)?;
    let changes = || {
        let mut buff = String::new();
        let mut buf_reader = BufReader::new(file);
        buf_reader.read_to_string(&mut buff)?;
        pager.push_str(&buff)?;
        Result::<(), Box<dyn std::error::Error>>::Ok(())
    };

    let pager = pager.clone();
    let res1 = thread::spawn(|| minus::dynamic_paging(pager));
    let res2 = changes();
    res1.join().unwrap()?;
    res2?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = args().collect();
    if arguments.len() < 2 {
        panic!("Not enough arguments");
    }
    let filename = arguments[1].clone();
    let output = minus::Pager::new();
    output.set_prompt(&filename)?;
    read_file(filename, output)
}
