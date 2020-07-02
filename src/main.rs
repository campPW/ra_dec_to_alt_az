use chrono::prelude::*;
use std::fmt;
use std::num::ParseFloatError;

fn main() {
    // use helper funcs to get ra-dec strs into decimal degrees
    let m1_ra = match to_decimal_degrees("05h 34m 31.94s", Coord::RA) {
        Ok(ra) => ra,
        Err(e) => panic!("Error occured when parsing right ascension : {:?}", e),
    };

    let m1_dec = match to_decimal_degrees("+22° 00′ 52.2″", Coord::DEC) {
        Ok(dec) => dec,
        Err(e) => panic!("An error occured when parsing declination: {:?}", e),
    };

    let m1 = AstroObject::new("M1 Crab Nebula (Supernova Remnant)", m1_ra, m1_dec);

    let location = GeoCoords {
        lat: 34.0522,
        long: 118.2437,
    };
    println!("{}", m1);
    println!("{:?}", m1.coords_as_alt_az(location));
}

struct GeoCoords {
    lat: f32,
    long: f32,
}
struct AstroObject<'a> {
    name: &'a str,
    right_ascension: f32,
    declination: f32,
}
#[derive(PartialEq)]
enum Coord {
    RA,
    DEC,
}
fn to_decimal_degrees(input: &str, coord_type: Coord) -> Result<f32, ParseFloatError> {
    let tokens: Vec<_> = input
        .split(&[' ', 'h', 'm', 's', '°', '′', '+', '-', '″'][..])
        .filter(|ch| !ch.is_empty())
        .collect();

    let hours_or_degrees: f32 = tokens[0].parse()?;
    let mins: f32 = tokens[1].parse()?;
    let secs: f32 = tokens[2].parse()?;

    let in_degrees = hours_or_degrees + (mins / 60.0) + (secs / 3600.0);

    if coord_type == Coord::RA {
        return Ok(in_degrees * 15.0);
    }

    Ok(in_degrees)
}

fn calculate_days_since_j2000() -> f32 {
    let j2000 = Utc.ymd(2000, 1, 1).and_hms(12, 0, 0);
    let now = Utc::now();
    let days_since = (now - j2000).num_seconds() as f32 / (24.0 * 3600.0);
    days_since
}

fn calculate_local_sidereal_time(days_j2000: f32, long: f32) -> f32 {
    let now = Utc::now();
    let fraction_of_hour = now.minute() as f32 / 60.0;
    let ut = now.hour() as f32 + fraction_of_hour;
    // this is an approximate formula for local sidereal time taken from linked article. See readme.md
    let local_siderial_time = (100.46 + 0.985647 * days_j2000 + long + 15.0 * ut + 360.0) % 360.0;
    local_siderial_time
}

fn calculate_alt_az(ha: f32, dec: f32, location: GeoCoords) -> (f32, f32) {
    let prelim_alt = (dec.to_radians().sin() * location.lat.to_radians().sin())
        + (dec.to_radians().cos() * location.lat.to_radians().cos() * ha.to_radians().cos());

    let alt = prelim_alt.asin().to_degrees();

    let prelim_az = (dec.to_radians().sin()
        - (alt.to_radians().sin() * location.lat.to_radians().sin()))
        / (alt.to_radians().cos() * location.lat.to_radians().cos());

    let prelim_az = prelim_az.acos().to_degrees();

    if ha.to_radians().sin().to_degrees() < 0.0 {
        let az = prelim_az;
        return (az, alt);
    }
    let az = 360.0 - prelim_az;
    (alt, az)
}

impl<'a> AstroObject<'a> {
    fn new(obj_name: &str, right_ascension: f32, declination: f32) -> AstroObject {
        AstroObject {
            name: obj_name,
            right_ascension: right_ascension,
            declination: declination,
        }
    }

    fn coords_as_alt_az(&self, location_info: GeoCoords) -> (f32, f32) {
        let days_j2000 = calculate_days_since_j2000();
        let local_sidereal_time = calculate_local_sidereal_time(days_j2000, location_info.long);
        let mut hour_angle = local_sidereal_time - self.right_ascension;
        if hour_angle < 0.0 {
            hour_angle += 360.0
        };
        let alt_az = calculate_alt_az(hour_angle, self.declination, location_info);
        alt_az
    }
}
impl<'a> fmt::Display for AstroObject<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write out the object name, it's RA, and DEC
        write!(
            f,
            "Name: {} \n Right Ascension: {} \n Declination: {} \n",
            self.name, self.right_ascension, self.declination
        )
    }
}
