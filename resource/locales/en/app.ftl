# Day Skies: English (default). https://daybrite.dev/docs/how-to/localization

app_title = Day Skies
app_tagline = Weather

# Cities (sidebar + hero)
city_san_francisco = San Francisco
city_new_york = New York
city_london = London
city_tokyo = Tokyo
city_sydney = Sydney

# Conditions (WMO weather-code families)
cond_clear = Clear
cond_partly_cloudy = Partly Cloudy
cond_cloudy = Cloudy
cond_fog = Fog
cond_drizzle = Drizzle
cond_rain = Rain
cond_snow = Snow
cond_thunder = Thunderstorms

# Forecast
weather_now = Now
weather_hilo = H:{ $high }° L:{ $low }°
weather_hourly = HOURLY FORECAST
weather_10day = 10-DAY FORECAST
weather_today = Today

day_sun = Sun
day_mon = Mon
day_tue = Tue
day_wed = Wed
day_thu = Thu
day_fri = Fri
day_sat = Sat

# Detail cards
detail_feels = Feels Like
detail_humidity = Humidity
detail_wind = Wind
detail_uv = UV Index
detail_sunrise = Sunrise
detail_sunset = Sunset
detail_pressure = Pressure

# States & attribution
weather_loading = Loading…
weather_error = Couldn't load weather
data_attribution = Weather data by Open-Meteo.com
data_mock = Mock data · screenshot mode

# Settings ▸ Weather (the General group's strings come from Day's catalog)
settings_units_section = Units
settings_unit_label = Temperature
unit_celsius = Celsius
unit_fahrenheit = Fahrenheit
settings_server_section = Weather server
settings_host_label = Host
settings_host_hint = Requests go to this Open-Meteo-compatible server — set your own proxy here, or leave the default.
settings_weather = Weather
settings_weather_subtitle = Units, weather server
settings_cities = Cities
settings_cities_count = { $count ->
    [one] 1 city
   *[other] { $count } cities
}
settings_refresh = Refresh

# Settings ▸ Cities (the city editor)
cities_list_section = Your cities
cities_add_section = Add a city
cities_preset_label = Pick a city
cities_add_preset = Add selected
cities_name_label = Name
cities_lat_label = Latitude
cities_lon_label = Longitude
cities_save = Save
cities_clear = Clear
cities_edit = Edit
cities_remove = Remove
cities_location_section = Current location
cities_use_location = Use current location
cities_my_location = My location
city_custom = { $name }
city_paris = Paris
city_berlin = Berlin
city_cairo = Cairo
city_singapore = Singapore
city_toronto = Toronto
cities_status_saved = Saved
cities_status_removed = Removed
cities_status_exists = Already on your list
cities_status_bad_coords = Enter a latitude from -90 to 90 and a longitude from -180 to 180
cities_status_need_name = Enter a name for the city
cities_status_locating = Finding your location…
cities_status_located = Location added
cities_status_permission = Location permission was not granted
cities_status_location_failed = Location failed: { $error }

# The language's own name (the Language row in Settings lists every bundled locale)
language_name = English

open_in_new_window = Open in New Window
