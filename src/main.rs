 #![allow(non_snake_case, non_upper_case_globals)]

extern crate time;
extern crate term;
extern crate rustc_serialize as serialize;
extern crate rustc_serialize;
extern crate hyper;


use std::io::Error;
use std::io::prelude::*;
use std::iter;
use std::iter::FromIterator;
use time::{strftime, strptime};
use serialize::json;
use hyper::{Client, Url};

static BASE_URL: &'static str = "https://api.worldweatheronline.com/free/v2/weather.ashx";
static KEY: &'static str = "a444bbde1001764c4634bc7079a7c";
static CELL_WIDTH: usize = 30;
static DAYS: usize = 3;


pub trait HasTerminalDisplayLength {
    fn len_on_term(&self) -> usize;
    fn fit_to_term_len(&self, new_len: usize) -> String;
}

impl HasTerminalDisplayLength for String {
    fn len_on_term(&self) -> usize {
        let mut ret = 0usize;
        let mut wait_for_color_mark_ends = false;

        for c in self.chars() {
            if c == '\u{1b}' && !wait_for_color_mark_ends {
                wait_for_color_mark_ends = true;
            } else if c == 'm' && wait_for_color_mark_ends {
                wait_for_color_mark_ends = false;
            } else {
                if !wait_for_color_mark_ends {
                    ret += 1;
                }
            }
        }
        ret
    }

    fn fit_to_term_len(&self, new_len: usize) -> String {
        let actual_len = self.len() + new_len - self.len_on_term();
        if self.len_on_term() < new_len  {
            String::from_iter(self
                              .chars()
                              .chain(iter::repeat(' ').take(actual_len - self.len())))
        } else {
            self[..actual_len].to_string()
        }
    }
}

fn wind_dir_to_icon(code: &str) -> &'static str {
    match code {
        "N"   => "\u{1b}[1m↓\u{1b}[0m",
	"NNE" => "\u{1b}[1m↓\u{1b}[0m",
	"NE"  => "\u{1b}[1m↙\u{1b}[0m",
	"ENE" => "\u{1b}[1m↙\u{1b}[0m",
	"E"   => "\u{1b}[1m←\u{1b}[0m",
	"ESE" => "\u{1b}[1m←\u{1b}[0m",
	"SE"  => "\u{1b}[1m↖\u{1b}[0m",
	"SSE" => "\u{1b}[1m↖\u{1b}[0m",
	"S"   => "\u{1b}[1m↑\u{1b}[0m",
	"SSW" => "\u{1b}[1m↑\u{1b}[0m",
	"SW"  => "\u{1b}[1m↗\u{1b}[0m",
	"WSW" => "\u{1b}[1m↗\u{1b}[0m",
	"W"   => "\u{1b}[1m→\u{1b}[0m",
	"WNW" => "\u{1b}[1m→\u{1b}[0m",
	"NW"  => "\u{1b}[1m↘\u{1b}[0m",
	"NNW" => "\u{1b}[1m↘\u{1b}[0m",
        _     => " "
    }
}

fn code_to_icon(code: i32) -> [&'static str; 5] {
    match code {
        113 => iconSunny,
        116 => iconPartlyCloudy,
        119 => iconCloudy,
        122 => iconVeryCloudy,
        143 => iconFog,
        176 => iconLightShowers,
        179 => iconLightSleetShowers,
        182 => iconLightSleet,
        185 => iconLightSleet,
        200 => iconThunderyShowers,
        227 => iconLightSnow,
        230 => iconHeavySnow,
        248 => iconFog,
        260 => iconFog,
        263 => iconLightShowers,
        266 => iconLightRain,
        281 => iconLightSleet,
        284 => iconLightSleet,
        293 => iconLightRain,
        296 => iconLightRain,
        299 => iconHeavyShowers,
        302 => iconHeavyRain,
        305 => iconHeavyShowers,
        308 => iconHeavyRain,
        311 => iconLightSleet,
        314 => iconLightSleet,
        317 => iconLightSleet,
        320 => iconLightSnow,
        323 => iconLightSnowShowers,
        326 => iconLightSnowShowers,
        329 => iconHeavySnow,
        332 => iconHeavySnow,
        335 => iconHeavySnowShowers,
        338 => iconHeavySnow,
        350 => iconLightSleet,
        353 => iconLightShowers,
        356 => iconHeavyShowers,
        359 => iconHeavyRain,
        362 => iconLightSleetShowers,
        365 => iconLightSleetShowers,
        368 => iconLightSnowShowers,
        371 => iconHeavySnowShowers,
        374 => iconLightSleetShowers,
        377 => iconLightSleet,
        386 => iconThunderyShowers,
        389 => iconThunderyHeavyRain,
        392 => iconThunderySnowShowers,
        395 => iconHeavySnowShowers, // ThunderyHeavySnow
        _   => iconUnknown

    }
}

static iconUnknown: [&'static str; 5] = [
		"    .-.      ",
		"     __)     ",
		"    (        ",
		"     `-’     ",
		"      •      "];
static iconSunny: [&'static str; 5] = [
		"\u{1b}[38;5;226m    \\   /    \u{1b}[0m",
		"\u{1b}[38;5;226m     .-.     \u{1b}[0m",
		"\u{1b}[38;5;226m  ― (   ) ―  \u{1b}[0m",
		"\u{1b}[38;5;226m     `-’     \u{1b}[0m",
		"\u{1b}[38;5;226m    /   \\    \u{1b}[0m"];
static iconPartlyCloudy: [&'static str; 5] = [
		"\u{1b}[38;5;226m   \\  /\u{1b}[0m      ",
		"\u{1b}[38;5;226m _ /\"\"\u{1b}[38;5;250m.-.    \u{1b}[0m",
		"\u{1b}[38;5;226m   \\_\u{1b}[38;5;250m(   ).  \u{1b}[0m",
		"\u{1b}[38;5;226m   /\u{1b}[38;5;250m(___(__) \u{1b}[0m",
		"             "];
static iconCloudy: [&'static str; 5] = [
		"             ",
		"\u{1b}[38;5;250m     .--.    \u{1b}[0m",
		"\u{1b}[38;5;250m  .-(    ).  \u{1b}[0m",
		"\u{1b}[38;5;250m (___.__)__) \u{1b}[0m",
		"             "];
static iconVeryCloudy: [&'static str; 5] = [
		"             ",
		"\u{1b}[38;5;240;1m     .--.    \u{1b}[0m",
		"\u{1b}[38;5;240;1m  .-(    ).  \u{1b}[0m",
		"\u{1b}[38;5;240;1m (___.__)__) \u{1b}[0m",
		"             "];
static iconLightShowers: [&'static str; 5] = [
		"\u{1b}[38;5;226m _`/\"\"\u{1b}[38;5;250m.-.    \u{1b}[0m",
		"\u{1b}[38;5;226m  ,\\_\u{1b}[38;5;250m(   ).  \u{1b}[0m",
		"\u{1b}[38;5;226m   /\u{1b}[38;5;250m(___(__) \u{1b}[0m",
		"\u{1b}[38;5;111m     ‘ ‘ ‘ ‘ \u{1b}[0m",
		"\u{1b}[38;5;111m    ‘ ‘ ‘ ‘  \u{1b}[0m"];
static iconHeavyShowers: [&'static str; 5] = [
		"\u{1b}[38;5;226m _`/\"\"\u{1b}[38;5;240;1m.-.    \u{1b}[0m",
		"\u{1b}[38;5;226m  ,\\_\u{1b}[38;5;240;1m(   ).  \u{1b}[0m",
		"\u{1b}[38;5;226m   /\u{1b}[38;5;240;1m(___(__) \u{1b}[0m",
		"\u{1b}[38;5;21;1m   ‚‘‚‘‚‘‚‘  \u{1b}[0m",
		"\u{1b}[38;5;21;1m   ‚’‚’‚’‚’  \u{1b}[0m"];
static iconLightSnowShowers: [&'static str; 5] = [
		"\u{1b}[38;5;226m _`/\"\"\u{1b}[38;5;250m.-.    \u{1b}[0m",
		"\u{1b}[38;5;226m  ,\\_\u{1b}[38;5;250m(   ).  \u{1b}[0m",
		"\u{1b}[38;5;226m   /\u{1b}[38;5;250m(___(__) \u{1b}[0m",
		"\u{1b}[38;5;255m     *  *  * \u{1b}[0m",
		"\u{1b}[38;5;255m    *  *  *  \u{1b}[0m"];
static iconHeavySnowShowers: [&'static str; 5] = [
		"\u{1b}[38;5;226m _`/\"\"\u{1b}[38;5;240;1m.-.    \u{1b}[0m",
		"\u{1b}[38;5;226m  ,\\_\u{1b}[38;5;240;1m(   ).  \u{1b}[0m",
		"\u{1b}[38;5;226m   /\u{1b}[38;5;240;1m(___(__) \u{1b}[0m",
		"\u{1b}[38;5;255;1m    * * * *  \u{1b}[0m",
		"\u{1b}[38;5;255;1m   * * * *   \u{1b}[0m"];
static iconLightSleetShowers: [&'static str; 5] = [
		"\u{1b}[38;5;226m _`/\"\"\u{1b}[38;5;250m.-.    \u{1b}[0m",
		"\u{1b}[38;5;226m  ,\\_\u{1b}[38;5;250m(   ).  \u{1b}[0m",
		"\u{1b}[38;5;226m   /\u{1b}[38;5;250m(___(__) \u{1b}[0m",
		"\u{1b}[38;5;111m     ‘ \u{1b}[38;5;255m*\u{1b}[38;5;111m ‘ \u{1b}[38;5;255m* \u{1b}[0m",
		"\u{1b}[38;5;255m    *\u{1b}[38;5;111m ‘ \u{1b}[38;5;255m*\u{1b}[38;5;111m ‘  \u{1b}[0m"];
static iconThunderyShowers: [&'static str; 5] = [
		"\u{1b}[38;5;226m _`/\"\"\u{1b}[38;5;250m.-.    \u{1b}[0m",
		"\u{1b}[38;5;226m  ,\\_\u{1b}[38;5;250m(   ).  \u{1b}[0m",
		"\u{1b}[38;5;226m   /\u{1b}[38;5;250m(___(__) \u{1b}[0m",
		"\u{1b}[38;5;228;5m    ⚡\u{1b}[38;5;111;25m‘ ‘\u{1b}[38;5;228;5m⚡\u{1b}[38;5;111;25m‘ ‘ \u{1b}[0m",
		"\u{1b}[38;5;111m    ‘ ‘ ‘ ‘  \u{1b}[0m"];
static iconThunderyHeavyRain: [&'static str; 5] = [
		"\u{1b}[38;5;240;1m     .-.     \u{1b}[0m",
		"\u{1b}[38;5;240;1m    (   ).   \u{1b}[0m",
		"\u{1b}[38;5;240;1m   (___(__)  \u{1b}[0m",
		"\u{1b}[38;5;21;1m  ‚‘\u{1b}[38;5;228;5m⚡\u{1b}[38;5;21;25m‘‚\u{1b}[38;5;228;5m⚡\u{1b}[38;5;21;25m‚‘   \u{1b}[0m",
		"\u{1b}[38;5;21;1m  ‚’‚’\u{1b}[38;5;228;5m⚡\u{1b}[38;5;21;25m’‚’   \u{1b}[0m"];
static iconThunderySnowShowers: [&'static str; 5] = [
		"\u{1b}[38;5;226m _`/\"\"\u{1b}[38;5;250m.-.    \u{1b}[0m",
		"\u{1b}[38;5;226m  ,\\_\u{1b}[38;5;250m(   ).  \u{1b}[0m",
		"\u{1b}[38;5;226m   /\u{1b}[38;5;250m(___(__) \u{1b}[0m",
		"\u{1b}[38;5;255m     *\u{1b}[38;5;228;5m⚡\u{1b}[38;5;255;25m *\u{1b}[38;5;228;5m⚡\u{1b}[38;5;255;25m * \u{1b}[0m",
		"\u{1b}[38;5;255m    *  *  *  \u{1b}[0m"];
static iconLightRain: [&'static str; 5] = [
		"\u{1b}[38;5;250m     .-.     \u{1b}[0m",
		"\u{1b}[38;5;250m    (   ).   \u{1b}[0m",
		"\u{1b}[38;5;250m   (___(__)  \u{1b}[0m",
		"\u{1b}[38;5;111m    ‘ ‘ ‘ ‘  \u{1b}[0m",
		"\u{1b}[38;5;111m   ‘ ‘ ‘ ‘   \u{1b}[0m"];
static iconHeavyRain: [&'static str; 5] = [
		"\u{1b}[38;5;240;1m     .-.     \u{1b}[0m",
		"\u{1b}[38;5;240;1m    (   ).   \u{1b}[0m",
		"\u{1b}[38;5;240;1m   (___(__)  \u{1b}[0m",
		"\u{1b}[38;5;21;1m  ‚‘‚‘‚‘‚‘   \u{1b}[0m",
		"\u{1b}[38;5;21;1m  ‚’‚’‚’‚’   \u{1b}[0m"];
static iconLightSnow: [&'static str; 5] = [
		"\u{1b}[38;5;250m     .-.     \u{1b}[0m",
		"\u{1b}[38;5;250m    (   ).   \u{1b}[0m",
		"\u{1b}[38;5;250m   (___(__)  \u{1b}[0m",
		"\u{1b}[38;5;255m    *  *  *  \u{1b}[0m",
		"\u{1b}[38;5;255m   *  *  *   \u{1b}[0m"];
static iconHeavySnow: [&'static str; 5] = [
		"\u{1b}[38;5;240;1m     .-.     \u{1b}[0m",
		"\u{1b}[38;5;240;1m    (   ).   \u{1b}[0m",
		"\u{1b}[38;5;240;1m   (___(__)  \u{1b}[0m",
		"\u{1b}[38;5;255;1m   * * * *   \u{1b}[0m",
		"\u{1b}[38;5;255;1m  * * * *    \u{1b}[0m"];
static iconLightSleet: [&'static str; 5] = [
		"\u{1b}[38;5;250m     .-.     \u{1b}[0m",
		"\u{1b}[38;5;250m    (   ).   \u{1b}[0m",
		"\u{1b}[38;5;250m   (___(__)  \u{1b}[0m",
		"\u{1b}[38;5;111m    ‘ \u{1b}[38;5;255m*\u{1b}[38;5;111m ‘ \u{1b}[38;5;255m*  \u{1b}[0m",
		"\u{1b}[38;5;255m   *\u{1b}[38;5;111m ‘ \u{1b}[38;5;255m*\u{1b}[38;5;111m ‘   \u{1b}[0m"];
static iconFog: [&'static str; 5] = [
		"             ",
		"\u{1b}[38;5;251m _ - _ - _ - \u{1b}[0m",
		"\u{1b}[38;5;251m  _ - _ - _  \u{1b}[0m",
		"\u{1b}[38;5;251m _ - _ - _ - \u{1b}[0m",
		"             "];

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct DataWrapper  {
    data: Data
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct Data  {
    current_condition: Vec<WeatherCondition>,
    request: Vec<Request>,
    weather: Vec<Weather>
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct WeatherCondition {
    cloudcover: i32,
    FeelsLikeC: i32,
    humidity: i32,
    precipMM: f32,
    weatherCode: i32,
    // FIXME: :(
    temp_C: Option<i32>,
    tempC: Option<i32>,
    time: Option<String>,
    chanceofrain: Option<i32>,
    observation_time: Option<String>,
    visibility: i32,
    weatherDesc: Vec<ValueWrapper>,
    winddir16Point: String,
    windspeedKmph: i32,
    WindGustKmph: Option<i32>
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct ValueWrapper {
    value: String
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct Request {
    query: String,
    // FIXME: waiting rust-nighty to enable serde compiler ext
    // type_: String
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct Weather {
    astronomy: Vec<Astronomy>,
    date: String,
    hourly: Vec<WeatherCondition>,
    maxtempC: i32,
    mintempC: i32,
    uvIndex: i32
}

impl Weather {
    fn print_day(&self, w: &mut Write) -> Result<(), Error> {
        let date_fmt = "┤ ".to_string() + strftime("%a %d. %b", strptime(&self.date, "%Y-%m-%d").as_ref().unwrap()).as_ref().unwrap() + " ├";
        try!(writeln!(w, "                                                       ┌─────────────┐                                                       "));
	try!(writeln!(w, "┌──────────────────────────────┬───────────────────────{}───────────────────────┬──────────────────────────────┐", date_fmt));
        try!(writeln!(w, "│           Morning            │             Noon      └──────┬──────┘    Evening            │            Night             │"));
        try!(writeln!(w, "├──────────────────────────────┼──────────────────────────────┼──────────────────────────────┼──────────────────────────────┤"));
        for line in self.format_day().iter() {
            try!(writeln!(w, "{}", line));
        }
        try!(writeln!(w, "└──────────────────────────────┴──────────────────────────────┴──────────────────────────────┴──────────────────────────────┘"));
        Ok(())
    }

    fn format_day(&self) -> Vec<String> {
        let mut ret = Vec::with_capacity(5);
        ret.extend(iter::repeat("|".to_string()).take(5));

        for h in self.hourly.iter() {
            let time = h.time.clone().unwrap();
            match time.as_ref() {
                "0" | "100" | "200" | "300" | "400" | "500" |
                "600" | "700" | "1400" | "1500" | "1600" | "2300" =>
                    continue,
                _                                                 => {
                    let cond_desc = h.format();

                    for (i, line) in ret.iter_mut().enumerate() {
                        let orig = line.clone();
                        *line = orig + &cond_desc[i] + "|";
                    }
                }
            }
        }
        ret
    }
}


#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct Astronomy {
    moonrise: String,
    moonset: String,
    sunrise: String,
    sunset: String
}


fn colorized_temp(temp: i32) -> String {
    let col = match temp {
        -15 | -14 | -13 => 27,
	-12 | -11 | -10 => 33,
	-9 | -8 | -7    => 39,
	-6 | -5 | -4    => 45,
	-3 | -2 | -1    => 51,
	0 | 1           => 50,
	2 | 3           => 49,
	4 | 5           => 48,
	6 | 7           => 47,
	8 | 9           => 46,
	10 | 11 | 12    => 82,
	13 | 14 | 15    => 118,
	16 | 17 | 18    => 154,
	19 | 20 | 21    => 190,
	22 | 23 | 24    => 226,
	25 | 26 | 27    => 220,
	28 | 29 | 30    => 214,
	31 | 32 | 33    => 208,
	34 | 35 | 36    => 202,
        _ if temp > 0   => 196,
        _               => 21
    };
    format!("\u{1b}[38;5;{:03}m{}\u{1b}[0m", col, temp)
}

fn colorized_wind(spd: i32) -> String {
    let col = match spd {
        1 | 2 | 3         => 82,
        4 | 5 | 6         => 118,
        7 | 8 | 9         => 154,
        10 | 11 | 12      => 190,
        13 | 14 | 15      => 226,
        16 | 17 | 18 | 19 => 220,
        20 | 21 | 22 | 23 => 214,
        24 | 25 | 26 | 27 => 208,
        28 | 29 | 30 | 31 => 202,
        _ if spd > 0      => 196,
        _                 => 46
    };
    format!("\u{1b}[38;5;{:03}m{}\u{1b}[0m", col, spd)
}


impl WeatherCondition {
    fn temp_in_C(&self) -> i32 {
        self.tempC.or(self.temp_C).unwrap()
    }

    fn format_visibility(&self) -> String {
        format!("{} {}            ", self.visibility, "km")[..15].to_string()
    }

    fn format_wind(&self) -> String {
        let windGustKmph = self.WindGustKmph.unwrap_or(0);
        if windGustKmph > self.windspeedKmph {
            format!("{} {} - {} {}      ",
                    wind_dir_to_icon(self.winddir16Point.as_ref()),
                    colorized_wind(self.windspeedKmph),
                    colorized_wind(windGustKmph),
                    "km/h")[..57].to_string()
        } else {
            format!("{} {} {}      ",
                    wind_dir_to_icon(self.winddir16Point.as_ref()),
                    colorized_wind(self.windspeedKmph),
                    "km/h").to_string()
        }
    }

    fn format_temp(&self) -> String {
        if self.FeelsLikeC < self.temp_in_C() {
            format!("{} - {} °C         ",
                    colorized_temp(self.FeelsLikeC),
                    colorized_temp(self.temp_in_C()))
        } else if self.FeelsLikeC > self.temp_in_C() {
            format!("{} - {} °C         ",
                    colorized_temp(self.temp_in_C()),
                    colorized_temp(self.FeelsLikeC))
        } else {
            format!("{} °C             ",
                    colorized_temp(self.FeelsLikeC))
        }
    }

    fn format_rain(&self) -> String {
        match self.chanceofrain {
            Some(ratio) =>
                format!("{:.1} {} | {}%        ", self.precipMM, "mm", ratio),
            None =>
                format!("{:.1} {}              ", self.precipMM, "mm")
        }
    }

    fn format(&self) -> Vec<String> {
        let icon = code_to_icon(self.weatherCode);
        vec![
            format!("{} {:-15.15}", icon[0], self.weatherDesc[0].value).fit_to_term_len(CELL_WIDTH),
            format!("{} {}", icon[1], self.format_temp()).fit_to_term_len(CELL_WIDTH),
            format!("{} {}", icon[2], self.format_wind()).fit_to_term_len(CELL_WIDTH),
            format!("{} {}", icon[3], self.format_visibility()).fit_to_term_len(CELL_WIDTH),
            format!("{} {}", icon[4], self.format_rain()).fit_to_term_len(CELL_WIDTH)]
    }
}


fn main() {
    let mut stdout = term::stdout().unwrap();

    let mut url = Url::parse(BASE_URL).unwrap();
    url.set_query_from_pairs(vec![("q", "Guangzhou"),
                                  ("key", KEY),
                                  ("format", "json")].iter().map(|&pair| pair));

    let mut client = Client::new();

    let mut res = client.get(url).send().unwrap();

    let mut buf = String::with_capacity(65535);
    match res.read_to_string(&mut buf) {
        Ok(_) => (),
        Err(e) => println!("err => {:?}", e)
    }

    let decoded: DataWrapper = json::decode(buf.as_ref()).unwrap();
    let data = decoded.data;

    println!("Weather for: {}\n\n", data.request[0].query);

    for line in data.current_condition[0].format() {
        println!("{}", line);
    }

    for w in data.weather.iter().take(DAYS) {
        w.print_day(&mut stdout).unwrap();
    }
}

#[test]
fn test_len_on_term() {
    let a = " \u{1b}[38;5;226m    \\   /    \u{1b}[0m Sunny             ".to_string();
    assert_eq!(a.len_on_term(), 33);
    let b = " \u{1b}[38;5;226m     .-.     \u{1b}[0m \u{1b}[38;5;226m24\u{1b}[0m - \u{1b}[38;5;220m25\u{1b}[0m \u{b0}C        ".to_string();
    assert_eq!(a.len_on_term(), 33);
}
