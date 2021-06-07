extern crate question; 

use image::{ImageError, io::Reader as ImageReader};
use core::panic;
use std::io::{Error};
use std::fs::{self, DirEntry};
use std::process::exit;
use question::{Answer, Question};
use structopt::StructOpt;
use std::fs::metadata;
use std::process::Command;

#[derive(Debug, StructOpt)]
struct Cli {
    /// Directory containing photos to transfer
    source_directory: String, 
    destination_directory: String, 
    error_directory: String, 
    /// New photo output type
    destination_file_type: String,
    /// Give time user to stop in between in batches
    #[structopt(short = "n")]
    batch_size: Option<isize>
}

fn transform_magick_fn(file_entry : &DirEntry, output_file_name : &String) {
  Command::new("convert")
  .arg(file_entry.path())
  .arg(output_file_name)
  .spawn()
  .expect("Unable to run convert executable...")
  .wait()
  .expect("Convert did not exit successfully...");

}
/* Loads in image located at 'file' path, and resaves it to results directory under the specified Image Format */
fn transform_image(file_entry : &DirEntry, destination_directory: &String, mut file_name: &str, dot_pos: usize, format : image::ImageFormat, transform_magick : bool) -> Result<(), ImageError> {

  let new_file_extension = match format {
    image::ImageFormat::Png => ".png",
    image::ImageFormat::Jpeg => ".jpeg",
    _ => panic!("I didn't implement this!")
  };

  /* this is to get rid of the old extension */
  file_name = &file_name[0..dot_pos];

  let output_file_name = format!("{}{}{}", destination_directory, file_name, new_file_extension);

  /* If heic, we transform differently than using ImageReader library. */
  if transform_magick {
    transform_magick_fn(file_entry, &output_file_name);  
  } else {
    let original_image = ImageReader::open(file_entry.path())?.decode()?;
    original_image.save_with_format(output_file_name, format)?;
  }

  Ok(())
}

/* copies the given file to the error directory. called on files with errors */
fn copy_file_error(source: &str, source_directory: &String, destination_directory: &String) {
  let from = format!("{}{}", source_directory, source);
  let to = format!("{}{}", destination_directory, source);
  fs::copy(from, to).expect("Unable to copy to error directory. This is a fatal error.");
}

fn transform_directory(source_directory: &String, destination_directory: &String, error_directory : &String, format: image::ImageFormat, batch_size : isize) -> Result<(), Error> {
  let paths = fs::read_dir(source_directory)?;

  /* count is going to check for the batch size */
  let mut count = 0;
  for path in paths {
    let curr_dir_entry = path?;
    let file_name_obj = curr_dir_entry.file_name();
    let file_name = file_name_obj.to_str().unwrap();

    let dot_pos_opt = file_name.find("."); 

    /* If you couldn't find a period, file extension is weird... */
    if dot_pos_opt.is_none() {
      println!("File {} did not have a period for file extension. Copying to error directory...", file_name);
      copy_file_error(file_name, source_directory, error_directory);
      continue;
    }
    /* Windows :Zone.Identifer is getting ignored here. Not adding this to error dir */
    if file_name.find(":").is_some() {
      continue;
    }

    let dot_pos = dot_pos_opt.unwrap();
    let mut file_extension = &file_name[dot_pos..];
    let lower = file_extension.to_ascii_lowercase();
    file_extension = lower.as_str();
    

    /* If file is png or heic, use magick which has better encoders for the cost of spawning a subprocess */
    let mut transform_magick = false;
    let is_supported = match file_extension {
      ".jpg" => true, 
      ".jpeg" => true, 
      ".png" => {transform_magick = true; true},
      ".heic" => {transform_magick = true; true},
      _ => false
    };

    if !is_supported {
     println!("File: {}, has an extension {} that is not supported. Moving to errors directory.", file_name, file_extension);
     copy_file_error(file_name, source_directory, error_directory);
     continue;
    }

    match transform_image(&curr_dir_entry, destination_directory, file_name, dot_pos, format, transform_magick) {
      Ok(()) => {}, 
      Err(error) => {
        println!("File {} could not convert properly. Error: {}. Moving to errors directory.", file_name, error);
        copy_file_error(file_name, source_directory, error_directory);
      }
    }

    /* finished another image successfully. increase batch size and confirm with user if batch size reached */
    count += 1;
    if count == batch_size {
      count = 0;
      let batch_continue = Question::new("Batch sized reached. Continue with yes. Stop transfer with no").confirm();

      match batch_continue {
        Answer::YES => {},
        Answer::NO => {
          println!("Stopping transfer...");
          break;
        }
        _ => unreachable!(),
      }
      println!("Continuing...");
    }
  }
  Ok(())
}

fn validate_directory(directory_path : &mut String) {
  if directory_path.find('/').is_none() {
    directory_path.push('/');
  }

  let metadata = metadata(&directory_path);
  if metadata.is_err() {
    println!("The directory: {} does not exist. Exiting...", directory_path);
    exit(1);
  }

  if !metadata.unwrap().is_dir() {
    println!("The destination path: {} is not a directory", directory_path);
  }
}

fn main() {


  let args = Cli::from_args();

  let mut source_directory = args.source_directory;
  let mut destination_directory = args.destination_directory;
  let mut error_directory = args.error_directory;

  /* Ensure these directories exist */
  validate_directory(&mut source_directory);
  validate_directory(&mut destination_directory);
  validate_directory(&mut error_directory);

  let destination_file_extension = args.destination_file_type;

  let output_format_type = match destination_file_extension.as_str() {
    ".jpeg" => image::ImageFormat::Jpeg,
    ".png" => image::ImageFormat::Png,
    _ => panic!("The given filetype: {} is not supported", destination_file_extension),
  };

  let batch_size = match args.batch_size {
    Some(val) => val, 
    None => -1,
  };

  println!("Beginning photo transfer to given format: {}. Batch size: {}", destination_file_extension, batch_size);
  let res = transform_directory(&source_directory, &destination_directory, &error_directory, output_format_type, batch_size);

  match res {
    Ok(()) => {}, 
    Err(error) => {
      println!("Error occured with transform directory.: {}. {}", source_directory, error)
    }
  }
  println!("Success!");
}

