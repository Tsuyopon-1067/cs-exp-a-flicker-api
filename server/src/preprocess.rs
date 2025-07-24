use crate::models::PhotoData;
use flate2::Compression;
use flate2::write::GzEncoder;
use indicatif::{ProgressBar, ProgressIterator};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

const MAX_PHOTOS_PER_TAG: usize = 100;
const OUTPUT_PATH: &str = "tag_photodata_map.bin";

pub fn preprocess(path1: &str, path2: &str) -> Result<(), Box<dyn Error>> {
    println!("Preprocessing with {} and {}", path1, path2);
    let tag_to_ids = create_tag_to_ids_map(path1)?;
    println!("Created map with {} unique tags", tag_to_ids.len());
    let id_to_photodata = create_id_to_photodata_map(path2)?;
    println!(
        "Created map with {} photo data entries",
        id_to_photodata.len()
    );

    let tag_to_photodata_gzip = create_tag_to_photodata_gzip_map(&tag_to_ids, &id_to_photodata)?;
    println!(
        "Created map with {} tags to photo data gzip",
        tag_to_photodata_gzip.len()
    );

    // マップをストレージに保存
    save_tag_photodata_map(&tag_to_photodata_gzip)?;
    println!("Successfully saved tag to photodata map to {}", OUTPUT_PATH);

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

fn create_tag_to_photodata_gzip_map(
    tag_to_ids: &HashMap<String, Vec<i64>>,
    id_to_photodata: &HashMap<i64, PhotoData>,
) -> Result<HashMap<String, Vec<u8>>, Box<dyn Error>> {
    let mut tag_to_photodata_gzip: HashMap<String, Vec<u8>> = HashMap::new();

    let pb = ProgressBar::new(tag_to_ids.len() as u64);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    for (tag, ids) in tag_to_ids.iter() {
        // 各タグに関連付けられたidからPhotoDataを取得
        let mut photos: Vec<&PhotoData> = ids
            .iter()
            .filter_map(|id| id_to_photodata.get(id))
            .collect();
        // 日付で新しい順（降順）にソート
        photos.sort_by(|a, b| b.date.cmp(&a.date));
        // 100件に切り捨て
        photos.truncate(MAX_PHOTOS_PER_TAG);
        // PhotoDataをJSON文字列に変換
        let photos_json = serde_json::to_string(&photos)?;
        // JSON文字列をGzip圧縮
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(photos_json.as_bytes())?;
        let compressed_bytes = encoder.finish()?;
        tag_to_photodata_gzip.insert(tag.clone(), compressed_bytes);

        pb.inc(1);
    }

    pb.finish_with_message("完了");
    Ok(tag_to_photodata_gzip)
}

fn save_tag_photodata_map(map: &HashMap<String, Vec<u8>>) -> Result<(), Box<dyn Error>> {
    // バイナリ形式でシリアライズして保存
    let serialized = bincode::serialize(map)?;
    std::fs::write(OUTPUT_PATH, serialized)?;

    Ok(())
}
