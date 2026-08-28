use std::time::{SystemTime, UNIX_EPOCH};

pub struct SolarSchedule {
    pub lat: f64,
    pub lon: f64,
    pub day_brightness: u32,
    pub night_brightness: u32,
    pub day_bluelight: u32,
    pub night_bluelight: u32,
    pub day_colortemp: Option<u32>,
    pub night_colortemp: Option<u32>,
}

impl SolarSchedule {
    pub fn new(
        lat: f64,
        lon: f64,
        day_b: u32,
        night_b: u32,
        day_bl: u32,
        night_bl: u32,
        day_ct: Option<u32>,
        night_ct: Option<u32>,
    ) -> Self {
        Self {
            lat,
            lon,
            day_brightness: day_b,
            night_brightness: night_b,
            day_bluelight: day_bl,
            night_bluelight: night_bl,
            day_colortemp: day_ct,
            night_colortemp: night_ct,
        }
    }

    pub fn is_daytime(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let seconds_in_day = now % 86400;
        let hour_utc = (seconds_in_day as f64) / 3600.0;

        let local_solar_hour = (hour_utc + (self.lon / 15.0)).rem_euclid(24.0);
        (6.0..18.0).contains(&local_solar_hour)
    }

    pub fn calculate_targets(&self) -> (u32, u32, Option<u32>) {
        if self.is_daytime() {
            (self.day_brightness, self.day_bluelight, self.day_colortemp)
        } else {
            (self.night_brightness, self.night_bluelight, self.night_colortemp)
        }
    }
}
