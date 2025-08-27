# point-cloud

Convert data from different sources to a point cloud.

Currently supported:

- Strava (.gpx, .gpx.gz, .fit.gz)
- Google Timeline (`location-history.json`)

## Usage

- `brew install tippecanoe`
- Clone repo
- put data in `./data`
- run `cargo run -r`

Creates `out.fgb`.

Convert that to a PMTiles file:

```sh
tippecanoe -o data/out.pmtiles --projection=EPSG:3857 --force --cluster-distance=1 -r1 data/out.fgb
```

You can now browse that file using [pmtiles.io](https://pmtiles.io/).
