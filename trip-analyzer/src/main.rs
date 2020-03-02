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
            .required(true)
        )
        // get_matches() メソッドを呼ぶとユーザーが与えた
        // コマンドライン引数がパースされる
        .get_matches();
    let infile = arg_matches.value_of("INFILE").unwrap();
    println!("INFILE: {}", infile);
}
