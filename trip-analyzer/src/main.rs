use clap::{App, Arg};

fn main() {
    println!("Hello, world!");
    let arg_matches = App::new("trip-analyzer")
        .version("1.0")
        .about("Analyze yellow cab trip records")
        // INFILE という名前のコマンドライン引数を登録
        .arg(Arg::with_name("INFILE")
            .help("Sets the input CSV file")
            .index(1) // 最初の引数
        )
        // get_matches() メソッドを呼ぶとユーザーが与えた
        // コマンドライン引数がパースされる
        .get_matches();
    // INFILE の文字列を表示。"{:?}"はデバッグ用文字列を表示
    println!("INFILE: {:?}", arg_matches.value_of("INFILE"));
}
