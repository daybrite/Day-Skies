import Foundation
import WidgetKit
import os

func publish_widget(snapshot: String) {
    guard let directory = FileManager.default.containerURL(
        forSecurityApplicationGroupIdentifier: "group.dev.daybrite.dayskies"
    ) else {
        Logger(subsystem: "dev.daybrite.dayskies", category: "widget")
            .error("Shared widget container is unavailable")
        return
    }
    let url = directory.appendingPathComponent("settings.json")
    let data = Data(snapshot.utf8)
    // Avoid consuming WidgetKit reload budget for an unchanged snapshot.
    guard (try? Data(contentsOf: url)) != data else { return }
    do {
        try data.write(to: url, options: .atomic)
        WidgetCenter.shared.reloadTimelines(ofKind: "DaySkiesWeather")
    } catch {
        Logger(subsystem: "dev.daybrite.dayskies", category: "widget")
            .error("Cannot publish widget snapshot: \(error.localizedDescription)")
    }
}
