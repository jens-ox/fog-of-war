# fog of war

![Screenshot](.github/screenshot.png)

Convert data from different sources to a fog of war-style map.

Currently supported:

- Strava (.gpx, .gpx.gz, .fit.gz)
- Google Timeline (`location-history.json`)

## Usage

First, prepare the data:

1. Collect all data and put it in the `data` directory.
2. Install [tippecanoe](https://github.com/felt/tippecanoe) (e.g. `brew install tippecanoe`).
3. Clone the repo.
4. Run `cargo run -r`.

Second, render the data. Inside the `ui` directory, do:

1. `bun install`
2. `bun run dev`

You should now be able to see an interactive map on `http://localhost:5173`.
