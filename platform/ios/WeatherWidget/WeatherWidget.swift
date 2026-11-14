import AppIntents
import SwiftUI
import WidgetKit

struct CityQuery: EntityQuery {
    func suggestedEntities() async throws -> [CityEntity] {
        (SharedSettings.read()?.cities ?? []).map { CityEntity(id: $0.id, name: $0.name) }
    }
    func entities(for identifiers: [String]) async throws -> [CityEntity] {
        let settings = SharedSettings.read()
        let missing = await MainActor.run {
            var resource = WidgetStrings.widget_unavailable
            resource.locale = Locale(identifier: settings?.locale ?? Locale.current.identifier)
            return String(localized: resource)
        }
        // Keep an unavailable selection's identity. Returning no entity can make App
        // Intents replace it with the default city, silently changing what the widget shows.
        return identifiers.map { id in
            CityEntity(id: id, name: settings?.city(id: id)?.name ?? missing)
        }
    }
    func defaultResult() async -> CityEntity? { try? await suggestedEntities().first }
}

struct WeatherEntry: TimelineEntry {
    let date: Date
    let settings: SharedSettings?
    let city: SavedCity?
    let forecast: Forecast?

    static func cached(cityID: String?) -> Self {
        let settings = SharedSettings.read()
        let city = settings?.city(id: cityID)
        let cached = city.flatMap { SharedStore.cached(city: $0, host: settings?.host ?? "") }
        let forecasts = [city?.weather, cached].compactMap { $0 }
        let forecast = city?.weather?.mock == true ? city?.weather : forecasts.max { $0.updated_at < $1.updated_at }
        if let city, let settings, let forecast, !forecast.mock {
            SharedStore.save(forecast, city: city, host: settings.host)
        }
        return Self(date: Date(), settings: settings, city: city, forecast: forecast)
    }
}

struct WeatherProvider: AppIntentTimelineProvider {
    func placeholder(in context: Context) -> WeatherEntry { .cached(cityID: nil) }
    func snapshot(for configuration: SelectCity, in context: Context) async -> WeatherEntry {
        .cached(cityID: configuration.city?.id)
    }
    func timeline(for configuration: SelectCity, in context: Context) async -> Timeline<WeatherEntry> {
        var entry = WeatherEntry.cached(cityID: configuration.city?.id)
        if let city = entry.city, let settings = entry.settings, entry.forecast?.mock != true,
           entry.forecast == nil || Date().timeIntervalSince1970 - entry.forecast!.updated_at > 900,
           let fresh = try? await WeatherService.fetch(city: city, host: settings.host) {
            SharedStore.save(fresh, city: city, host: settings.host)
            entry = WeatherEntry(date: Date(), settings: settings, city: city, forecast: fresh)
        }
        // WidgetKit budgets refreshes. The app also requests a reload after data/settings edits.
        // A later entry hides the hourly strip even if the system delays the next network refresh.
        var entries = [entry]
        if let forecast = entry.forecast {
            let expiry = Date(timeIntervalSince1970: forecast.updated_at + 7200)
            if expiry > entry.date {
                entries.append(WeatherEntry(date: expiry, settings: entry.settings,
                    city: entry.city, forecast: forecast))
            }
        }
        return Timeline(entries: entries, policy: .after(entry.date.addingTimeInterval(1800)))
    }
}

struct WeatherView: View {
    @Environment(\.widgetFamily) private var family
    let entry: WeatherEntry

    private var locale: Locale { Locale(identifier: entry.settings?.locale ?? Locale.current.identifier) }
    private func text(_ resource: LocalizedStringResource) -> String {
        var resource = resource
        resource.locale = locale
        return String(localized: resource)
    }
    private func temperature(_ celsius: Double) -> String {
        let formatter = MeasurementFormatter()
        formatter.locale = locale
        formatter.unitOptions = .providedUnit
        formatter.unitStyle = .short
        formatter.numberFormatter.maximumFractionDigits = 0
        let measurement = Measurement(value: celsius, unit: UnitTemperature.celsius)
        return formatter.string(from: entry.settings?.fahrenheit == true ? measurement.converted(to: .fahrenheit) : measurement)
    }
    private var link: URL? {
        guard let id = entry.city?.id else { return URL(string: "dayskies://") }
        return URL(string: "dayskies://\(id.addingPercentEncoding(withAllowedCharacters: .urlPathAllowed) ?? "")")
    }
    private var background: LinearGradient {
        let day = entry.forecast?.is_day ?? true
        let clear = ["Clear", "PartlyCloudy"].contains(entry.forecast?.family ?? "")
        return LinearGradient(colors: day && clear ? [Color(red: 0.08, green: 0.35, blue: 0.68), Color(red: 0.22, green: 0.59, blue: 0.82)] :
            [Color(red: 0.09, green: 0.15, blue: 0.28), Color(red: 0.28, green: 0.38, blue: 0.53)], startPoint: .topLeading, endPoint: .bottomTrailing)
    }

    private func weatherIcon(_ weather: Forecast) -> some View {
        Image(systemName: Conditions.symbol(weather.family, day: weather.is_day))
            .symbolRenderingMode(.multicolor).font(.system(size: 28)).accessibilityHidden(true)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 5) {
            Text(entry.city?.name ?? text(WidgetStrings.app_title))
                .font(.headline).lineLimit(1).minimumScaleFactor(0.7)
            if let w = entry.forecast {
                HStack(alignment: .center, spacing: 12) {
                    VStack(alignment: .leading, spacing: 4) {
                        HStack {
                            Text(temperature(w.temp))
                                .font(.system(size: 34, weight: .light, design: .rounded))
                                .minimumScaleFactor(0.65).lineLimit(1)
                            if family == .systemSmall {
                                Spacer(minLength: 2)
                                weatherIcon(w)
                            }
                        }
                        Text(text(Conditions.label(w.family))).font(.caption).lineLimit(1)
                        HStack(spacing: 6) {
                            Label(temperature(w.high), systemImage: "arrow.up")
                                .accessibilityLabel(text(WidgetStrings.widget_high) + " " + temperature(w.high))
                            Label(temperature(w.low), systemImage: "arrow.down")
                                .accessibilityLabel(text(WidgetStrings.widget_low) + " " + temperature(w.low))
                        }.font(.caption2).lineLimit(1).minimumScaleFactor(0.7)
                    }.frame(maxWidth: .infinity, alignment: .leading)
                    if family == .systemMedium {
                        if entry.date.timeIntervalSince1970 - w.updated_at < 7200 {
                            HStack(spacing: 6) {
                                ForEach(Array(w.hourly.prefix(4).enumerated()), id: \.offset) { _, hour in
                                    VStack(spacing: 8) {
                                        Text(hour.label ?? text(WidgetStrings.weather_now)).font(.system(size: 10))
                                        Image(systemName: Conditions.symbol(hour.family, day: hour.is_day))
                                            .symbolRenderingMode(.multicolor).accessibilityLabel(text(Conditions.label(hour.family)))
                                        Text(temperature(hour.temp)).font(.system(size: 10)).minimumScaleFactor(0.6).lineLimit(1)
                                    }.frame(maxWidth: .infinity)
                                }
                            }.frame(maxWidth: .infinity)
                        } else {
                            weatherIcon(w).frame(maxWidth: .infinity)
                        }
                    }
                }
                Spacer(minLength: 0)
                if w.mock {
                    Text(text(WidgetStrings.data_mock)).font(.system(size: 9)).lineLimit(1)
                } else {
                    HStack(spacing: 3) {
                        Text(text(WidgetStrings.widget_updated))
                        Text(Date(timeIntervalSince1970: w.updated_at), style: .relative)
                    }.font(.system(size: 9)).opacity(0.8).lineLimit(1)
                }
            } else {
                Spacer()
                Image(systemName: "cloud.sun.fill").font(.largeTitle).accessibilityHidden(true)
                Text(text(entry.city == nil ? WidgetStrings.widget_empty : WidgetStrings.widget_unavailable)).font(.caption)
                Spacer()
            }
        }
        .foregroundStyle(.white)
        .containerBackground(for: .widget) { background }
        .widgetURL(link)
        .environment(\.locale, locale)
        .environment(\.layoutDirection, locale.language.characterDirection == .rightToLeft ? .rightToLeft : .leftToRight)
    }
}

@main
struct DaySkiesWidget: Widget {
    var body: some WidgetConfiguration {
        AppIntentConfiguration(kind: "DaySkiesWeather", intent: SelectCity.self, provider: WeatherProvider()) {
            WeatherView(entry: $0)
        }
        .configurationDisplayName(WidgetStrings.widget_title)
        .description(WidgetStrings.widget_description)
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}
