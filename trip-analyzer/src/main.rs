use clap::{App, Arg};
use std::error::Error;
use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use hdrhistogram::Histogram;

// NaiveDateTimeは長いのでDTという別名を定義
// chrono::NaiveDateTimeはタイムゾーンなしの日時型
type DT = NaiveDateTime;
// ついでにResult型の別名を定義する
type AppResult<T> = Result<T, Box<dyn Error>>;

fn parse_datetime(s: &str) -> AppResult<DT> {
    DT::parse_from_str(s, "%Y-%m-%d %H:%M:%S").map_err(|e| e.into())
}

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

// serde_jsonでJSON文字列を生成するためにSerializeを自動導出する
#[derive(Debug, Serialize)]
struct RecordCounts {
    read: u32,
    matched: u32,
    skipped: u32,
}

impl Default for RecordCounts {
    fn default() -> Self {
        Self {
            read: 0,
            matched: 0,
            skipped: 0,
        }
    }
}

// CSVファイルのパスを引数に取り、データを分析する
fn analyze(infile: &str) -> AppResult<String> {
    // CSVリーダーを作る。失敗したときは「?」後置演算子の働きにより、
    // analyze() 関数からすぐにリターンし、処理の失敗を表すResult::Errを返す
    let mut reader = csv::Reader::from_path(infile)?;

    let mut rec_counts = RecordCounts::default();
    let mut hist = DurationHistograms::new()?;
    for (i, result) in reader.deserialize().enumerate() {
        // どの型にデシリアライズするかをdeserialize()メソッドに
        // 教えるために、trip 変数に型アノテーションをつける
        let trip: Trip = result?;
        rec_counts.read += 1;
        // 最初の10行だけ表示する
        if rec_counts.read <= 10 {
            println!("{:?}", trip);
        }

        if is_in_midtown(trip.pickup_loc) && is_jfk_airport(trip.dropoff_loc) {
            let pickup = parse_datetime(&trip.pickup_datetime)?;
            if is_weekday(pickup) {
                rec_counts.matched += 1;
                let dropoff = parse_datetime(&trip.dropoff_datetime)?;
                hist.record_duration(pickup, dropoff)
                    .unwrap_or_else(|e| {
                        eprintln!("WARN: {} - {}. Skipped: {:?}", i + 2, e, trip);
                        rec_counts.skipped += 1;
                    });
            }
        }
    }
    println!("{:?}", rec_counts);
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

fn is_in_midtown(loc: LocId) -> bool {
    // LocId の配列を作る
    let locations = [90, 100, 161, 162, 163, 164, 186, 230, 234];
    // 配列に対してバイナリサーチする。
    // locと同じ値があれば Ok(値のインデックス) が返る
    locations.binary_search(&loc).is_ok()
}

// ロケーションIDがJFK国際空港ならtrueを返す
fn is_jfk_airport(loc: LocId) -> bool {
    loc == 132
}

fn is_weekday(datetime: DT) -> bool {
    // 月:1, 火:2, ... 金:5, 土:6, 日:7
    datetime.weekday().number_from_monday() <= 5
}

// DurationHistogramsをタプル構造体として定義する
// この構造体はHistogramを24個持つことで、1時間刻みの時間帯ごとに
// 所要時間のヒストグラムデータを追跡する。
// Vec<T> 型は配列の一種
struct DurationHistograms(Vec<Histogram<u64>>);
// 関連関数やメソッドを実装するためにimplブロックを作る
impl DurationHistograms {
    // Histograms を初期化する関連関数。記録する上限値を引数に取る
    fn new() -> AppResult<Self> {
        let lower_bound = 1;
        let upper_bound = 3 * 60 *60;
        let hist = Histogram::new_with_bounds(lower_bound, upper_bound, 3)
            .map_err(|e| format!("{:?}", e))?;
        // histの値を24回複製してVec<T>配列に収集する
        let histograms = std::iter::repeat(hist).take(24).collect();
        Ok(Self(histograms))
    }

    fn record_duration(&mut self, pickup: DT, dropoff: DT) -> AppResult<()> {
        // 所要時間を秒で求める。結果は i64 型になるが as u64 で u64 型に変換
        let duration = (dropoff - pickup).num_seconds() as u64;

        // 20分未満はエラーにする
        if duration < 20 * 60 {
            Err(format!("duration secs {} is too short.", duration).into())
        } else {
            let hour = pickup.hour() as usize;
            // タプル構造体の最初のフィールドの名前は0になるので、
            // self.0 でVec<Histogram>にアクセスできる。さらに個々の
            // Histogramにアクセスするには [インデックス] で
            // その要素のインデックスを指定する
            self.0[hour]
                // Histogram の record() メソッドで所要時間を記録する
                .record(duration)
                // このメソッドはHistogramの作成時に設定した上限(upper_bound)
                // を超えているとErr(RecordError)を返すので、map_err() で
                // Err(String)に変換する
                .map_err(|e| {
                    format!("duration secs {} is too long. {:?}", duration, e)
                        .into()
                })
        }
    }
}
