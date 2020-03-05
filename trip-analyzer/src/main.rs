use clap::{App, Arg};
use std::error::Error;
use serde::Deserialize;

type LocId = u16;
#[derive(Debug, Deserialize)]
struct Trip {
    // rename アトリビュートでフィールド名と
    // CSV のカラム名を結びつける
    #[serde(rename = "tpep_pickup_datetime")]
    pickup_datetime: String,
    #[serde(rename = "tpep_dropoff_datetime")]
    dropoff_datetime: String,
    #[serde(rename = "PULocationID")]
    pickup_loc: LocId,
    #[serde(rename = "DOLocationID")]
    dropoff_loc: LocId,
}

// CSVファイルのパスを引数に取り、データを分析する
fn analyze(infile: &str) -> Result<String, Box<dyn Error>> {
    // CSVリーダーを作る。失敗したときは「?」後置演算子の働きにより、
    // analyze() 関数からすぐにリターンし、処理の失敗を表すResult::Errを返す
    let mut reader = csv::Reader::from_path(infile)?;

    let mut rec_counts = 0;
    for result in reader.deserialize() {
        // どの型にデシリアライズするかをdeserialize()メソッドに
        // 教えるために、trip 変数に型アノテーションをつける
        let trip: Trip = result?;
        rec_counts += 1;
        // 最初の10行だけ表示する
        if rec_counts <= 10 {
            println!("{:?}", trip);
        }
    }
    // 読み込んだレコード数を表示する
    println!("Total {} records read.", rec_counts);
    Ok(String::default())
}

fn main() {
    let arg_matches = App::new("trip-analyzer")
        .version("1.0")
        .about("Analyze yellow cab trip records")
        // INFILE という名前のコマンドライン引数を登録
        .arg(Arg::with_name("INFILE")
            .help("Sets the input CSV file")
            .index(1) // 最初の引数
            .required(true)
        )
        // get_matches() メソッドを呼ぶとユーザーが与えた
        // コマンドライン引数がパースされる
        .get_matches();
    let infile = arg_matches.value_of("INFILE").unwrap();
    match analyze(infile) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
