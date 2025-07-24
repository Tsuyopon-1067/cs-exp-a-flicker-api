use indicatif::{ProgressBar, ProgressIterator};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn preprocess(path1: &str, path2: &str) -> Result<(), Box<dyn Error>> {
    println!("Preprocessing with {} and {}", path1, path2);
    let tag_to_ids = create_tag_to_ids_map(path1)?;
    println!("Created map with {} unique tags", tag_to_ids.len());
    Ok(())
}

fn count_lines(path: &str) -> Result<u64, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count() as u64)
}

fn create_tag_to_ids_map(path: &str) -> Result<HashMap<String, Vec<i64>>, Box<dyn Error>> {
    let num_lines = count_lines(path)?;
    let file = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    let mut tag_to_ids: HashMap<String, Vec<i64>> = HashMap::new();
    let pb = ProgressBar::new(num_lines);

    for result in rdr.deserialize().progress_with(pb) {
        let (id, tag): (i64, String) = result?;
        tag_to_ids.entry(tag).or_default().push(id);
    }

    Ok(tag_to_ids)
}
