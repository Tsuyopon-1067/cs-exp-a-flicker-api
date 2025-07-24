use crate::models::PhotoData;
use indicatif::{ProgressBar, ProgressIterator};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn preprocess(path1: &str, path2: &str) -> Result<(), Box<dyn Error>> {
    println!("Preprocessing with {} and {}", path1, path2);
    let tag_to_ids = create_tag_to_ids_map(path1)?;
    println!("Created map with {} unique tags", tag_to_ids.len());
    let id_to_photodata = create_id_to_photodata_map(path2)?;
    println!(
        "Created map with {} photo data entries",
        id_to_photodata.len()
    );

    // "dog" タグで検索
    let search_tag = "dog";
    if let Some(ids) = tag_to_ids.get(search_tag) {
        let photos: Vec<&PhotoData> = ids
            .iter()
            .filter_map(|id| id_to_photodata.get(id))
            .collect();

        if photos.is_empty() {
            println!("No photos found for tag '{}'", search_tag);
        } else {
            let json_output = serde_json::to_string_pretty(&photos)?;
            println!("Photos for tag '{}':", search_tag);
            println!("{}", json_output);
        }
    } else {
        println!("Tag '{}' not found.", search_tag);
    }

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

fn create_id_to_photodata_map(path: &str) -> Result<HashMap<i64, PhotoData>, Box<dyn Error>> {
    let num_lines = count_lines(path)?;
    let file = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    let mut id_to_photodata: HashMap<i64, PhotoData> = HashMap::new();
    let pb = ProgressBar::new(num_lines);

    for result in rdr.deserialize().progress_with(pb) {
        let (id, date, lat, lon, url): (i64, String, f64, f64, String) = result?;
        let photodata: PhotoData = PhotoData::new(&date, lat, lon, &url)?;
        id_to_photodata.insert(id, photodata);
    }

    Ok(id_to_photodata)
}
