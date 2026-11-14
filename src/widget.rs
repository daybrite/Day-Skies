//! Versioned, atomic hand-off to the iOS widget. The app owns settings.json; the
//! extension owns its separate forecast cache, so their writes cannot race.
use crate::{cities, resource_for, settings, weather};
use serde_json::{Value, json};

fn forecast(w: &weather::Weather) -> Value {
    json!({
        "updated_at": w.updated_at, "mock": w.source == weather::DataSource::Mock,
        "temp": w.temp, "high": w.high, "low": w.low,
        "family": format!("{:?}", w.family), "is_day": w.is_day,
        "hourly": w.hourly.iter().take(6).map(|h| json!({
            "label": h.hour_label, "temp": h.temp,
            "family": format!("{:?}", h.family), "is_day": h.is_day,
        })).collect::<Vec<_>>()
    })
}

pub fn install() {
    // Other platforms have no widget and need no serialization work.
    if !cfg!(target_os = "ios") {
        return;
    }
    day::reactive::Effect::new(|| {
        let cities = cities::cities().get();
        let entries: Vec<_> = cities
            .iter()
            .map(|city| {
                let weather = match resource_for(city).signal().get() {
                    day::reactive::Load::Ready(w) => Some(forecast(&w)),
                    _ => None,
                };
                json!({"id": city.id, "name": cities::title(city).format(),
                "latitude": city.latitude, "longitude": city.longitude, "weather": weather})
            })
            .collect();
        let snapshot = json!({"version": 1, "cities": entries,
            "fahrenheit": settings::unit() == settings::Unit::Fahrenheit,
            "locale": day::locale().get(), "host": settings::host()});
        publish_widget(&snapshot.to_string());
    });
}

day_bridge::bridge! {
    #[day_bridge::declare]
    extern "day" { fn publish_widget(snapshot: &str); }

    #[day_bridge::impl(swift, platforms = [ios], src = "platform/ios/WidgetPublisher.swift")]

    #[day_bridge::impl(rust, platforms = [other])]
    fn publish_widget(_snapshot: &str) {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn snapshot_keeps_celsius_and_marks_synthetic_data() {
        let city = crate::cities::City {
            id: "sf".into(),
            name: String::new(),
            latitude: 0.0,
            longitude: 0.0,
        };
        let w = crate::weather::mock(&city);
        let value = super::forecast(&w);
        assert_eq!(value["temp"], w.temp);
        assert_eq!(value["mock"], true);
        assert_eq!(value["updated_at"], w.updated_at);
        assert_eq!(value["hourly"].as_array().unwrap().len(), 6);
    }
}
