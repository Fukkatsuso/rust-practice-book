use ansi_term::Style;
use chrono::{Datelike, Local, NaiveDate};
use clap::{App, Arg};
use itertools::izip;
use std::{error::Error, str::FromStr};

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const LINE_WIDTH: usize = 22;

#[derive(Debug)]
pub struct Config {
    month: Option<u32>,
    year: i32,
    today: NaiveDate,
}

type MyResult<T> = Result<T, Box<dyn Error>>;

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("calr")
        .version("0.1.0")
        .author("Fukkatsuso <fukkatsuso.git+github@gmail.com>")
        .about("Rust cal")
        .arg(
            Arg::with_name("year")
                .value_name("YEAR")
                .help("Year (1-9999)"),
        )
        .arg(
            Arg::with_name("month")
                .short("m")
                .value_name("MONTH")
                .help("Month name or number (1-12)"),
        )
        .arg(
            Arg::with_name("current_year")
                .short("y")
                .long("year")
                .takes_value(false)
                .help("Show whole current year")
                .conflicts_with_all(&["month", "year"]),
        )
        .get_matches();

    let mut month = matches.value_of("month").map(parse_month).transpose()?;
    let mut year = matches.value_of("year").map(parse_year).transpose()?;
    let today = Local::today();

    if matches.is_present("current_year") {
        month = None;
        year = Some(today.year());
    } else if month.is_none() && year.is_none() {
        month = Some(today.month());
        year = Some(today.year());
    }

    Ok(Config {
        month,
        year: year.unwrap_or_else(|| today.year()),
        today: today.naive_local(),
    })
}

fn parse_int<T: FromStr>(val: &str) -> MyResult<T> {
    val.parse()
        .map_err(|_| format!("Invalid integer \"{}\"", val).into())
}

fn parse_year(year: &str) -> MyResult<i32> {
    parse_int(year).and_then(|num| {
        // 1 <= n && n <= 9999
        if (1..=9999).contains(&num) {
            Ok(num)
        } else {
            Err(format!("year \"{}\" not in the range 1 through 9999", year).into())
        }
    })
}

fn parse_month(month: &str) -> MyResult<u32> {
    match parse_int(month) {
        Ok(num) => {
            if (1..=12).contains(&num) {
                Ok(num)
            } else {
                Err(format!("month \"{}\" not in the range 1 through 12", num).into())
            }
        }
        _ => {
            let lower = &month.to_lowercase();
            let matches: Vec<_> = MONTH_NAMES
                .iter()
                .enumerate()
                .filter_map(|(i, name)| {
                    if name.to_lowercase().starts_with(lower) {
                        Some(i + 1)
                    } else {
                        None
                    }
                })
                .collect();

            if matches.len() == 1 {
                Ok(matches[0] as u32)
            } else {
                Err(format!("Invalid month \"{}\"", month).into())
            }
        }
    }
}

pub fn run(config: Config) -> MyResult<()> {
    match config.month {
        Some(month) => {
            let lines = format_month(config.year, month, true, config.today);
            println!("{}", lines.join("\n"));
        }
        None => {
            println!("{:>32}", config.year);
            let months: Vec<_> = (1..=12)
                .into_iter()
                .map(|month| format_month(config.year, month, false, config.today))
                .collect();

            for (i, chunk) in months.chunks(3).enumerate() {
                if let [m1, m2, m3] = chunk {
                    for lines in izip!(m1, m2, m3) {
                        println!("{}{}{}", lines.0, lines.1, lines.2);
                    }
                    if i < 3 {
                        println!();
                    }
                }
            }
        }
    }
    Ok(())
}

fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
    let first = NaiveDate::from_ymd(year, month, 1);
    let mut days: Vec<String> = (1..first.weekday().number_from_sunday())
        .into_iter()
        .map(|_| "  ".to_string())
        .collect();

    let is_today = |day: u32| year == today.year() && month == today.month() && day == today.day();

    let last = last_day_in_month(year, month);
    days.extend((first.day()..=last.day()).into_iter().map(|num| {
        let fmt = format!("{:>2}", num);
        if is_today(num) {
            Style::new().reverse().paint(fmt).to_string()
        } else {
            fmt
        }
    }));

    let month_name = MONTH_NAMES[month as usize - 1];
    let mut lines = Vec::with_capacity(8);
    lines.push(format!(
        "{:^20}  ",
        if print_year {
            format!("{} {}", month_name, year)
        } else {
            month_name.to_string()
        }
    ));

    lines.push("Su Mo Tu We Th Fr Sa  ".to_string());

    for week in days.chunks(7) {
        lines.push(format!(
            "{:width$}  ",
            week.join(" "),
            width = LINE_WIDTH - 2
        ));
    }

    while lines.len() < 8 {
        lines.push(" ".repeat(LINE_WIDTH));
    }

    lines

    // ボツ

    // let mut res = vec![];

    // let header = format!(
    //     "{:^20}",
    //     if print_year {
    //         format!("{} {}", MONTH_NAMES[(month - 1) as usize], year)
    //     } else {
    //         MONTH_NAMES[(month - 1) as usize].to_string()
    //     }
    // );
    // res.push(header);
    // res.push("Su Mo Tu We Th Fr Sa".to_string());

    // let mut date = NaiveDate::from_ymd(year, month, 1);
    // let last_date = last_day_in_month(year, month);
    // while date.le(&last_date) {
    //     if date.day() == 1 || date.weekday() == Weekday::Sun {
    //         res.push("".to_string());
    //     }
    //     res.last().unwrap().push_str(format!("{}"));
    //     date = date.checked_add_days(Days::new(1)).unwrap();
    // }

    // res
}

fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    let (y, m) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd(y, m, 1).pred()

    // ボツ
    // let days = vec![31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    // let leap = (year % 400 == 0) || ((year % 4 == 0) && (year % 100 != 0));
    // if leap && month == 2 {
    //     NaiveDate::from_ymd(year, month, 29)
    // } else {
    //     NaiveDate::from_ymd(year, month, days[(month - 1) as usize])
    // }
}

#[cfg(test)]
mod tests {
    use super::{format_month, last_day_in_month, parse_int, parse_month, parse_year};
    use chrono::NaiveDate;

    #[test]
    fn test_parse_int() {
        // 正の整数をusizeとして解析する
        let res = parse_int::<usize>("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1usize);

        // 負の整数をi32として解析する
        let res = parse_int::<i32>("-1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), -1i32);

        // 数字以外の文字列を解析すると失敗する
        let res = parse_int::<i64>("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid integer \"foo\"");
    }

    #[test]
    fn test_parse_year() {
        let res = parse_year("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1i32);

        let res = parse_year("9999");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 9999i32);

        let res = parse_year("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"0\" not in the range 1 through 9999"
        );

        let res = parse_year("10000");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"10000\" not in the range 1 through 9999"
        );

        let res = parse_year("foo");
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_month() {
        let res = parse_month("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("12");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12u32);

        let res = parse_month("jan");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"0\" not in the range 1 through 12"
        );

        let res = parse_month("13");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"13\" not in the range 1 through 12"
        );

        let res = parse_month("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid month \"foo\"");
    }

    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd(0, 1, 1);
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);

        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);

        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd(2021, 4, 7);
        assert_eq!(format_month(2021, 4, true, today), april_hl);
    }

    #[test]
    fn test_last_day_in_month() {
        assert_eq!(last_day_in_month(2020, 1), NaiveDate::from_ymd(2020, 1, 31));
        assert_eq!(last_day_in_month(2020, 2), NaiveDate::from_ymd(2020, 2, 29));
        assert_eq!(last_day_in_month(2020, 4), NaiveDate::from_ymd(2020, 4, 30));
    }
}
