use crate::models::{PhotoData, ResponseData};
use ahash::AHashMap;
use flate2::Compression;
use flate2::write::GzEncoder;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

const MAX_PHOTOS_PER_TAG: usize = 100;
const OUTPUT_PATH: &str = "tag_photodata_map.bin";

pub fn preprocess(path1: &str, path2: &str) -> Result<(), Box<dyn Error>> {
    println!("Preprocessing with {} and {}", path1, path2);
    // tagから[id]へのmapを作成
    let tag_to_ids = create_tag_to_ids_map(path1)?;
    println!("Created map with {} unique tags", tag_to_ids.len());
    // idからphotodataへのmapを作成
    let id_to_photodata = create_id_to_photodata_map(path2)?;
    println!(
        "Created map with {} photo data entries",
        id_to_photodata.len()
    );

    // 2つのmapを組み合わせて tagからgzip([photodata])へのmapを作成
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

fn create_tag_to_ids_map(path: &str) -> Result<AHashMap<String, Vec<i64>>, Box<dyn Error>> {
    let num_lines = count_lines(path)?;
    let file = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    let mut tag_to_ids: AHashMap<String, Vec<i64>> = AHashMap::new();
    let pb = ProgressBar::new(num_lines);

    for result in rdr.deserialize() {
        let (id, tag): (i64, String) = result?;
        tag_to_ids.entry(tag).or_default().push(id);
        pb.inc(1);
    }
    pb.finish_with_message("完了");

    Ok(tag_to_ids)
}

fn create_id_to_photodata_map(path: &str) -> Result<AHashMap<i64, PhotoData>, Box<dyn Error>> {
    let num_lines = count_lines(path)?;
    let file = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    let mut id_to_photodata: AHashMap<i64, PhotoData> = AHashMap::new();
    let pb = ProgressBar::new(num_lines);

    for result in rdr.deserialize() {
        let (id, date, lat, lon, url): (i64, String, f64, f64, String) = result?;
        let photodata: PhotoData = PhotoData::new(lat, lon, &date, &url)?;
        id_to_photodata.insert(id, photodata);
        pb.inc(1);
    }
    pb.finish_with_message("完了");

    Ok(id_to_photodata)
}

fn create_tag_to_photodata_gzip_map(
    tag_to_ids: &AHashMap<String, Vec<i64>>,
    id_to_photodata: &AHashMap<i64, PhotoData>,
) -> Result<AHashMap<String, Vec<u8>>, Box<dyn Error>> {
    let pb = ProgressBar::new(tag_to_ids.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    let tag_to_photodata_gzip: AHashMap<String, Vec<u8>> = tag_to_ids
        .par_iter()
        .map(|(tag, ids)| {
            // 各タグに関連付けられたidからPhotoDataを取得
            let mut photos: Vec<PhotoData> = ids
                .iter()
                .filter_map(|id| id_to_photodata.get(id))
                .cloned()
                .collect();
            // 日付で新しい順（降順）にソート
            photos.sort_by(|a, b| b.date.cmp(&a.date));
            // 100件に切り捨て
            photos.truncate(MAX_PHOTOS_PER_TAG);
            // レスポンスのデータ構造に変換
            let response_data = ResponseData {
                tag: tag.clone(),
                results: photos.clone(),
            };
            // レスポンスデータをJSON文字列に変換
            let response_json = serde_json::to_string(&response_data).unwrap();
            // JSON文字列をGzip圧縮
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(response_json.as_bytes()).unwrap();
            let compressed_bytes = encoder.finish().unwrap();

            pb.inc(1);
            (tag.clone(), compressed_bytes)
        })
        .collect::<Vec<(String, Vec<u8>)>>()
        .into_iter()
        .collect();

    pb.finish_with_message("完了");
    Ok(tag_to_photodata_gzip)
}

fn save_tag_photodata_map(map: &AHashMap<String, Vec<u8>>) -> Result<(), Box<dyn Error>> {
    // バイナリ形式でシリアライズして保存
    let serialized = bincode::serialize(map)?;
    std::fs::write(OUTPUT_PATH, serialized)?;

    Ok(())
}
