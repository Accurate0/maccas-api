pub struct SensorDataRequest {}

#[derive(Debug, serde::Deserialize)]
pub struct SensorDataResponse {
    pub sensor_data: String,
}

impl SensorDataRequest {
    pub fn path() -> String {
        "sensor-data".to_string()
    }
}
