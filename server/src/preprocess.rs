use std::collections::HashSet;
use std::error::Error;
use std::fs::File;

pub fn preprocess(path: &str) -> Result<(), Box<dyn Error>> {
    process_csv(path)?;
    Ok(())
}

fn process_csv(path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut tags = HashSet::new();

    for result in rdr.records() {
        let record = result?;
        if let Some(tag) = record.get(1) {
            tags.insert(tag.to_string());
        }
    }

    let unique_tags: Vec<String> = tags.into_iter().collect();
    println!("Unique tags: {:?}", unique_tags.len());

    Ok(())
}
