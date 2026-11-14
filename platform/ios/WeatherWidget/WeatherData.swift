import Foundation
import CryptoKit

struct SharedSettings: Decodable, Sendable {
    let version: Int
    let cities: [SavedCity]
    let fahrenheit: Bool
    let locale: String
    let host: String

    func city(id: String?) -> SavedCity? {
        guard let id else { return cities.first }
        return cities.first { $0.id == id }
    }

    static func read() -> Self? {
        guard let url = SharedStore.directory?.appendingPathComponent("settings.json"),
              let data = try? Data(contentsOf: url),
              let settings = try? JSONDecoder().decode(Self.self, from: data),
              settings.version == 1 else { return nil }
        return settings
    }
}

struct SavedCity: Decodable, Sendable {
    let id: String
    let name: String
    let latitude: Double
    let longitude: Double
    let weather: Forecast?
}

struct Forecast: Codable, Sendable {
    let updated_at: Double
    let mock: Bool
    let temp: Double
    let high: Double
    let low: Double
    let family: String
    let is_day: Bool
    let hourly: [ForecastHour]
}

struct ForecastHour: Codable, Sendable {
    let label: String?
    let temp: Double
    let family: String
    let is_day: Bool
}

enum SharedStore {
    static var directory: URL? {
        FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: "group.dev.daybrite.dayskies")
    }

    static func cacheURL(city: SavedCity, host: String) -> URL? {
        // Coordinates and server are part of the identity: edits must never reuse old weather.
        let key = "\(city.id)|\(city.latitude)|\(city.longitude)|\(host)"
        let digest = SHA256.hash(data: Data(key.utf8)).map { String(format: "%02x", $0) }.joined()
        return directory?.appendingPathComponent("forecast-\(digest).json")
    }

    static func cached(city: SavedCity, host: String) -> Forecast? {
        guard let url = cacheURL(city: city, host: host), let data = try? Data(contentsOf: url) else { return nil }
        return try? JSONDecoder().decode(Forecast.self, from: data)
    }

    static func save(_ forecast: Forecast, city: SavedCity, host: String) {
        guard let url = cacheURL(city: city, host: host), let data = try? JSONEncoder().encode(forecast) else { return }
        try? data.write(to: url, options: .atomic)
    }
}

enum Conditions {
    static func family(_ code: Int) -> String {
        switch code {
        case 0, 1: "Clear"
        case 2: "PartlyCloudy"
        case 45, 48: "Fog"
        case 51, 53, 55, 56, 57: "Drizzle"
        case 61, 63, 65, 66, 67, 80, 81, 82: "Rain"
        case 71, 73, 75, 77, 85, 86: "Snow"
        case 95, 96, 99: "Thunder"
        default: "Cloudy"
        }
    }

    static func symbol(_ family: String, day: Bool) -> String {
        switch family {
        case "Clear": day ? "sun.max.fill" : "moon.stars.fill"
        case "PartlyCloudy": day ? "cloud.sun.fill" : "cloud.moon.fill"
        case "Fog": "cloud.fog.fill"
        case "Drizzle": "cloud.drizzle.fill"
        case "Rain": "cloud.rain.fill"
        case "Snow": "cloud.snow.fill"
        case "Thunder": "cloud.bolt.rain.fill"
        default: "cloud.fill"
        }
    }

    static func label(_ family: String) -> LocalizedStringResource {
        switch family {
        case "Clear": WidgetStrings.cond_clear
        case "PartlyCloudy": WidgetStrings.cond_partly_cloudy
        case "Fog": WidgetStrings.cond_fog
        case "Drizzle": WidgetStrings.cond_drizzle
        case "Rain": WidgetStrings.cond_rain
        case "Snow": WidgetStrings.cond_snow
        case "Thunder": WidgetStrings.cond_thunder
        default: WidgetStrings.cond_cloudy
        }
    }
}

/// WidgetKit invokes this independently of the host app. Values remain Celsius.
enum WeatherService {
    struct Response: Decodable {
        struct Current: Decodable {
            let time: Double
            let temperature_2m: Double
            let weather_code: Int
            let is_day: Int
        }
        struct Hourly: Decodable {
            let time: [Double]
            let temperature_2m: [Double]
            let weather_code: [Int]
            let is_day: [Int]
        }
        struct Daily: Decodable {
            let temperature_2m_max: [Double]
            let temperature_2m_min: [Double]
        }
        let current: Current
        let hourly: Hourly
        let daily: Daily
        let utc_offset_seconds: Int

        func forecast(now: Date) throws -> Forecast {
            guard let high = daily.temperature_2m_max.first, let low = daily.temperature_2m_min.first,
                  hourly.time.count == hourly.temperature_2m.count,
                  hourly.time.count == hourly.weather_code.count,
                  hourly.time.count == hourly.is_day.count else { throw URLError(.cannotParseResponse) }
            let formatter = DateFormatter()
            formatter.locale = Locale(identifier: "en_US_POSIX")
            formatter.timeZone = TimeZone(secondsFromGMT: utc_offset_seconds)
            formatter.dateFormat = "HH:mm"
            let hours = hourly.time.indices.filter { hourly.time[$0] >= current.time }.prefix(5).map { i in
                ForecastHour(label: formatter.string(from: Date(timeIntervalSince1970: hourly.time[i])),
                    temp: hourly.temperature_2m[i], family: Conditions.family(hourly.weather_code[i]),
                    is_day: hourly.is_day[i] == 1)
            }
            return Forecast(updated_at: now.timeIntervalSince1970, mock: false,
                temp: current.temperature_2m, high: high, low: low,
                family: Conditions.family(current.weather_code), is_day: current.is_day == 1, hourly: hours)
        }
    }

    static func fetch(city: SavedCity, host: String) async throws -> Forecast {
        guard var url = URLComponents(string: "https://\(host)"), url.host != nil,
              url.user == nil, url.password == nil, url.path.isEmpty,
              url.query == nil, url.fragment == nil else { throw URLError(.badURL) }
        url.path = "/v1/forecast"
        url.queryItems = [
            URLQueryItem(name: "latitude", value: String(city.latitude)),
            URLQueryItem(name: "longitude", value: String(city.longitude)),
            URLQueryItem(name: "current", value: "temperature_2m,weather_code,is_day"),
            URLQueryItem(name: "hourly", value: "temperature_2m,weather_code,is_day"),
            URLQueryItem(name: "daily", value: "temperature_2m_max,temperature_2m_min"),
            URLQueryItem(name: "timezone", value: "auto"),
            URLQueryItem(name: "timeformat", value: "unixtime"),
            URLQueryItem(name: "forecast_days", value: "2"),
            URLQueryItem(name: "temperature_unit", value: "celsius")
        ]
        guard let endpoint = url.url else { throw URLError(.badURL) }
        var request = URLRequest(url: endpoint)
        request.timeoutInterval = 15
        let (data, response) = try await URLSession.shared.data(for: request)
        guard let http = response as? HTTPURLResponse, (200..<300).contains(http.statusCode) else {
            throw URLError(.badServerResponse)
        }
        return try JSONDecoder().decode(Response.self, from: data).forecast(now: Date())
    }
}
