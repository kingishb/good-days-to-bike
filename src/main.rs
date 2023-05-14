use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::prelude::*;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NOAAForecast {
    #[serde(rename = "@context")]
    pub context: (String, Context),
    #[serde(rename = "type")]
    pub type_field: String,
    pub geometry: Geometry,
    pub properties: Properties,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    #[serde(rename = "@version")]
    pub version: String,
    pub wx: String,
    pub geo: String,
    pub unit: String,
    #[serde(rename = "@vocab")]
    pub vocab: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Geometry {
    #[serde(rename = "type")]
    pub type_field: String,
    pub coordinates: Vec<Vec<Vec<f64>>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    pub updated: String,
    pub units: String,
    pub forecast_generator: String,
    pub generated_at: String,
    pub update_time: String,
    pub valid_times: String,
    pub elevation: Elevation,
    pub periods: Vec<Period>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Elevation {
    pub unit_code: String,
    pub value: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Period {
    pub number: i64,
    pub name: String,
    pub start_time: String,
    pub end_time: String,
    pub is_daytime: bool,
    pub temperature: i64,
    pub temperature_unit: String,
    pub temperature_trend: Value,
    pub probability_of_precipitation: ProbabilityOfPrecipitation,
    pub dewpoint: Dewpoint,
    pub relative_humidity: RelativeHumidity,
    pub wind_speed: String,
    pub wind_direction: String,
    pub icon: String,
    pub short_forecast: String,
    pub detailed_forecast: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbabilityOfPrecipitation {
    pub unit_code: String,
    pub value: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dewpoint {
    pub unit_code: String,
    pub value: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelativeHumidity {
    pub unit_code: String,
    pub value: i64,
}

fn parse_wind_speed(s: &str) -> u8 {
    s.split(" ")
        .next()
        .unwrap_or("255")
        .parse::<u8>()
        .unwrap_or(u8::MAX)
}

#[derive(Debug)]
struct TimePeriod {
    start_time: String,
    end_time: String,
    temp: i64,
    probability_of_precipitation: i64,
    max_wind_speed: u8,
}

impl TimePeriod {
    fn pretty(&self) -> String {

            let start = DateTime::parse_from_rfc3339(&self.start_time).unwrap().format("%A, %B %d %I:%M%p");
            let end = DateTime::parse_from_rfc3339(&self.end_time).unwrap().format("%I:%M%p");
            
        format!("{0} - {1}, temp: {2} F, precipitation {3}%, wind speed: {4} mph", start, end, self.temp, self.probability_of_precipitation, self.max_wind_speed)
    }
}

fn coalesce(periods: Vec<&Period>) -> Vec<TimePeriod> {
    let mut tp: Vec<TimePeriod> = vec![];
    for cur in periods.into_iter() {
        if tp.len() > 0 {
            let mut prev = tp.pop().unwrap();
            if prev.end_time == cur.start_time {
                prev.end_time = cur.end_time.clone();
                prev.temp = std::cmp::max(prev.temp, cur.temperature);
                prev.probability_of_precipitation = std::cmp::max(
                    prev.probability_of_precipitation,
                    cur.probability_of_precipitation.value,
                );
                prev.max_wind_speed =
                    std::cmp::max(prev.max_wind_speed, parse_wind_speed(&cur.wind_speed));
                tp.push(prev);
            } else {
                tp.push(prev);
                tp.push(TimePeriod {
                    start_time: cur.start_time.clone(),
                    end_time: cur.end_time.clone(),
                    temp: cur.temperature.clone(),
                    probability_of_precipitation: cur.probability_of_precipitation.value.clone(),
                    max_wind_speed: parse_wind_speed(&cur.wind_speed),
                });
            }
        } else {
            tp.push(TimePeriod {
                start_time: cur.start_time.clone(),
                end_time: cur.end_time.clone(),
                temp: cur.temperature.clone(),
                probability_of_precipitation: cur.probability_of_precipitation.value.clone(),
                max_wind_speed: parse_wind_speed(&cur.wind_speed),
            });
        }
    }
    return tp;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.weather.gov/gridpoints/LWX/97,75/forecast/hourly";
    let client = reqwest::blocking::Client::new();
    let resp: NOAAForecast = client
        .get(url)
        .header("User-Agent", "brian")
        .send()?
        .json::<NOAAForecast>()?;

    let mut periods = vec![];
    for period in resp.properties.periods.iter() {
        if period.is_daytime && period.probability_of_precipitation.value < 25 {
            if period.temperature >= 50 && period.temperature <= 65 {
                let wind_speed = parse_wind_speed(&period.wind_speed);
                if wind_speed < 13 {
                    periods.push(period);
                }
            } else if period.temperature > 65 && period.temperature <= 83 {
                let wind_speed = parse_wind_speed(&period.wind_speed);
                if wind_speed <= 18 {
                    periods.push(period);
                }
            }
        }
    }
    for period in coalesce(periods).iter() {
        println!("{}", period.pretty());
    }

    Ok(())
}
