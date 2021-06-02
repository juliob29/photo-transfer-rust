extern crate question; 

use image::{ImageError, io::Reader as ImageReader};
use core::panic;
use std::io::{Error};
use std::fs;
use question::{Answer, Question};


/* Loads in image located at 'file' path, and resaves it to results directory under the specified Image Format */
fn transform_image(mut path_name : &str, mut file_name: &str, dot_pos: usize, format : image::ImageFormat) -> Result<(), ImageError> {
  let original_image = ImageReader::open(path_name)?.decode()?;

  let new_file_extension = match format {
    image::ImageFormat::Png => ".png",
    image::ImageFormat::Jpeg => ".jpeg",
    _ => panic!("I didn't implement this!")
  };

  /* this is to get rid of the old extension */
  file_name = &file_name[0..dot_pos];

  let output_file_name = format!("results/{}{}", file_name, new_file_extension);
  original_image.save_with_format(output_file_name, format)?;

  Ok(())
}

/* copies the given file to the error directory. called on files with errors */
fn copy_file_error(source: &str) {
  let from = format!("photos/{}", source);
  let to = format!("errors/{}", source);
  fs::copy(from, to).expect("Unable to copy to error directory. This is a fatal error.");
}

fn transform_directory(directory: &String, format: image::ImageFormat, batch_size : isize) -> Result<(), Error> {
  let paths = fs::read_dir(directory)?;

  /* count is going to check for the batch size */
  let mut count = 0;
  for path in paths {
    let curr_dir_entry = path?;
    let curr_path = curr_dir_entry.path();

    let curr_path_str = curr_path.to_str().unwrap();

    /* this slices away the photos/ part of the file name, hence the use of a constant */
    let file_name = &curr_path_str[7..];

    let dot_pos_opt = file_name.find("."); 

    /* If you couldn't find a period, file extension is weird... */
    if dot_pos_opt.is_none() {
      println!("File {} did not have a period for file extension. Copying to error directory...", curr_path_str);
      copy_file_error(file_name);
      continue;
    }

    /* Windows :Zone.Identifer is getting ignored here. Not adding this to error dir */
    if file_name.find(":").is_some() {
      continue;
    }

    let dot_pos = dot_pos_opt.unwrap();
    let file_extension = &file_name[dot_pos..];

    let is_supported = match file_extension {
      ".jpg" => true, 
      ".png" => true, 
      ".jpeg" => true, 
      _ => false
    };

    if !is_supported {
     println!("File: {}, has an extension {} that is not supported. Moving to errors directory.", curr_path_str, file_extension);
     copy_file_error(file_name);
     continue;
    }

    match transform_image(curr_path_str, file_name, dot_pos, format) {
      Ok(()) => {}, 
      Err(error) => {
        println!("File {} could not convert properly. Error: {}. Moving to errors directory.", curr_path_str, error);
        copy_file_error(file_name);
      }
    }

    /* finished another image successfully. increase batch size and confirm with user if batch size reached */
    count += 1;

    if count == batch_size {
      count = 0;
      let batch_continue = Question::new("Batch sized reached. Continue with yes. Stop transfer with no").confirm();

      match batch_continue {
        Answer::YES => println!("Continuing..."),
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

fn main() {
  let directory = String::from("photos/");

  loop {
    let answer = Question::new("Welcome to Photo Transferer. Ensure that your photos are in the 'photos' directory. Ready?").confirm();
    if let Answer::YES = answer {
      break;
    }
  }

  let destination_file_extension_answer = Question::new("What would you like to convert the photos to?").ask().unwrap();

  let destination_file_extension = match destination_file_extension_answer {
    Answer::RESPONSE(resp) => resp,
    _ => unreachable!(),
  };

  let output_format_type = match destination_file_extension.as_str() {
    ".jpeg" => image::ImageFormat::Jpeg,
    ".png" => image::ImageFormat::Png,
    _ => panic!("The given filetype: {} is not supported", destination_file_extension),
  };

  let batch_question = Question::new("We can prepare your photos in batches.
We can prepare a given number of photos, and stop and allow you to tell 
us when to proceed. Would you like to use photo batches?").confirm();
  
  
  let mut batch_size = -1;
  if let Answer::YES = batch_question {
    let size = Question::new("Enter a batch size").ask().unwrap();
    if let Answer::RESPONSE(resp) = size {
      batch_size = resp.parse::<isize>().unwrap();
    }
  }

  let res = transform_directory(&directory, output_format_type, batch_size);

  match res {
    Ok(()) => {}, 
    Err(error) => {
      println!("Error occured with transform directory.: {}. {}", directory, error)
    }
  }

  println!("Success!");
}

