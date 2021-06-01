use image::{ImageError, io::Reader as ImageReader};
use std::io::{Error, ErrorKind};
use std::io::Cursor;
use std::fs;

/* Loads in image located at 'file' path, and resaves it to results directory under the specified Image Format */
fn transform_image(mut file : &str, format : image::ImageFormat) -> Result<(), ImageError> {
  let original_image = ImageReader::open(file)?.decode()?;
  let slash_pos = file.find("/");
  let dot_pos = file.find(".").unwrap(); // already checked before calling this fn

  /* Get rid of the slash that might come when having the pictures in the samples/ directory. */
  if slash_pos.is_some() {
    file = &file[slash_pos.unwrap() + 1..dot_pos];
  }

  let file_extension = match format {
    image::ImageFormat::Png => ".png",
    image::ImageFormat::Jpeg => ".jpeg",
    _ => panic!("I didn't implement this!")
  };

  let output_file_name = format!("results/{}{}", file, file_extension);
  original_image.save_with_format(output_file_name, format)?;
  Ok(())
}

fn transform_directory(directory: &String, format: image::ImageFormat) -> Result<(), Error> {
  let paths = fs::read_dir(directory)?;

  for path in paths {
    let curr_path = path?.path();
    let curr_path_str = curr_path.to_str().unwrap();

    let period_pos_opt = curr_path_str.find("."); 

    /* If you couldn't find a period, file extension is weird... */
    if period_pos_opt.is_none() {
      println!("File {} did not have a period for file extension. Moving to error directory...", curr_path_str);
      continue;
    }

    /* Windows :Zone.Identifer is getting ignored here */
    if curr_path_str.find(":").is_some() {
      continue;
    }

    let period_pos_val = period_pos_opt.unwrap();
    let file_extension = &curr_path_str[period_pos_val..];

    let is_supported = match file_extension {
      ".jpg" => true, 
      ".png" => true, 
      ".jpeg" => true, 
      _ => false
    };

    if !is_supported {
     println!("File: {}, has an extension {} that is not supported. Moving to errors directory.", curr_path_str, file_extension);
     continue;
    }

    match transform_image(curr_path_str, format) {
      Ok(()) => {}, 
      Err(error) => {
        println!("File {} could not convert properly. Error: {}. Moving to errors directory.", curr_path_str, error);
      }
    }
  }

  Ok(())

}



fn main() {
  let directory = String::from("samples/");
  let res = transform_directory(&directory, image::ImageFormat::Jpeg);

  match res {
    Ok(()) => {}, 
    Err(error) => {
      println!("Error occured with image at path: {}. {}", directory, error)
    }
  }
}

