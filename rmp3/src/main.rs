extern crate simplemad;
use std::env;
use std::process;
use std::fs::File;
use simplemad::Decoder;
use std::time::Duration;
fn main() {

    // Collect the environment parameters.
    // args[0] is the program name
    // args[1] is the file name
    let args: Vec<String> = env::args().collect();

    // If there are less than 2 arguments then exit
    if args.len() < 2 {
        println!("Not enough arguments");
        process::exit(1);
    } else {
        println!("Filename:{}", args[1]);
		// Get a handle to file
		 let file = File::open(&args[1]);
         if file.is_ok() {
             //Decode headers
             #[derive(Debug)]
             let duration = Decoder::decode_headers(file.unwrap()).unwrap().filter_map(|r| {
             match r {
                     Ok(f) => Some(f.duration),
                     Err(_) => None,
             }}).fold(Duration::new(0, 0), |acc, dtn| acc + dtn);

            println!("{:?}",duration);
         } else {
             // Print the error and exit
             println!("{}", file.err().unwrap());
             process::exit(1);
         }
    }
}
