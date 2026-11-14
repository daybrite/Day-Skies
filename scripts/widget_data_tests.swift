// Synthetic protocol fixtures, not shipped UI or bundled resources.
// Run: scripts/test-widget.sh
import Foundation

@main
struct WidgetDataTests {
    static func main() async throws {
        for locale in ["en", "fr", "ar", "zh-CN"] {
            let source = try String(contentsOfFile: "resource/locales/\(locale)/app.ftl", encoding: .utf8)
            let generated = try Data(contentsOf: URL(fileURLWithPath:
                "platform/ios/WeatherWidget/Resources/\(locale).lproj/Localizable.strings"))
            let messages = try PropertyListSerialization.propertyList(from: generated, format: nil) as! [String: String]
            for (key, value) in messages {
                let line = source.components(separatedBy: "\n").first { $0.hasPrefix(key + " = ") }!
                precondition(value == String(line.dropFirst(key.count + 3)), "Native translation changed: \(locale)/\(key)")
            }
        }
        let data = Data(#"""
        {"version":1,"fahrenheit":true,"locale":"fr","host":"weather.example",
         "cities":[{"id":"paris","name":"Paris","latitude":48.86,"longitude":2.35,
         "weather":{"updated_at":1000,"mock":false,"temp":18,"high":20,"low":10,
         "family":"Clear","is_day":true,"hourly":[{"label":null,"temp":18,"family":"Clear","is_day":true}]}}]}
        """#.utf8)
        let settings = try JSONDecoder().decode(SharedSettings.self, from: data)
        precondition(settings.fahrenheit && settings.locale == "fr")
        precondition(settings.city(id: nil)?.id == "paris")
        precondition(settings.city(id: "removed") == nil, "A removed city must not silently show another city")
        precondition(settings.city(id: "paris")?.weather?.temp == 18)
        precondition(settings.city(id: "paris")?.weather?.hourly.first?.label == nil)

        let api = Data(#"""
        {"utc_offset_seconds":3600,"current":{"time":3600,"temperature_2m":12.5,"weather_code":95,"is_day":0},
         "hourly":{"time":[0,3600,7200],"temperature_2m":[10,12.5,13],"weather_code":[0,95,63],"is_day":[0,0,1]},
         "daily":{"temperature_2m_max":[18],"temperature_2m_min":[6]}}
        """#.utf8)
        let forecast = try JSONDecoder().decode(WeatherService.Response.self, from: api)
            .forecast(now: Date(timeIntervalSince1970: 4000))
        precondition(forecast.family == "Thunder" && !forecast.is_day && !forecast.mock)
        precondition(forecast.updated_at == 4000 && forecast.temp == 12.5)
        precondition(forecast.hourly.count == 2 && forecast.hourly.first?.label == "02:00")
        precondition(forecast.hourly.last?.family == "Rain" && forecast.hourly.last?.is_day == true)

        let malformed = String(decoding: api, as: UTF8.self).replacingOccurrences(of: "[10,12.5,13]", with: "[10]")
        do {
            _ = try JSONDecoder().decode(WeatherService.Response.self, from: Data(malformed.utf8)).forecast(now: Date())
            preconditionFailure("Mismatched hourly arrays must be rejected without indexing past the end")
        } catch { /* Expected malformed response. */ }
        precondition(Conditions.family(999) == "Cloudy")
        precondition(Conditions.symbol("Clear", day: false) == "moon.stars.fill")
        if let path = ProcessInfo.processInfo.environment["WIDGET_SNAPSHOT"] {
            let live = try JSONDecoder().decode(SharedSettings.self, from: Data(contentsOf: URL(fileURLWithPath: path)))
            precondition(live.version == 1 && !live.cities.isEmpty)
            precondition(live.cities.contains { $0.weather != nil })
            print("Decoded the running app's shared snapshot")
        }
        if ProcessInfo.processInfo.environment["WIDGET_TEST_LIVE"] == "1" {
            let fresh = try await WeatherService.fetch(city: settings.cities[0], host: "api.open-meteo.com")
            precondition(!fresh.mock && !fresh.hourly.isEmpty)
            print("Independent widget fetch succeeded: \(fresh.temp) Celsius")
        }
        print("Widget tests passed: four native catalogs, snapshot, selection, units, forecast, timezone, malformed response, conditions")
    }
}
