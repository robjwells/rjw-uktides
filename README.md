## `rjw-uktides`

A Rust library and command-line tool to fetch and parse the tide predictions
and tide prediction station list available from the UK Hydrographic Office
(UKHO) [EasyTide] service, which covers about 700 tide prediction stations
across Great Britain and Ireland.

[EasyTide]: https://easytide.admiralty.co.uk/

The library provides functions for getting the URLs that return a list of
stations and tide predictions for a particular station, and functions that
parse the JSON data returned from these URLs. The library itself does not
depend on any particular HTTP client and does not perform network IO itself.

The `tides` command-line tool uses the library to provide a simple way to look up
tide prediction data without having to write your own programme.

### What information is available?

In the `TidePredictions` struct, the most interesting field is `tidal_event_list`,
which includes the dates and times of high and low tides. There is also information
on the times of lunar phases, and a "footer note" which is typically safety-related.
Most stations (nearly 90%) also list the predicted height at 30-minute intervals.

### Library example usage

The following code block is a full example of fetching all the available tide stations,
looking up tide predictions, and printing the times of high and low tides.

```rust
// Fetch the list of all stations.
let url = rjw_uktides::stations_list_url();
let body = ureq::get(url.as_str()).call()?.into_body();
let stations = Vec<Station> = rjw_uktides::stations_from_reader(body.into_reader())?;

// Fetch tide predictions for the first station (probably Braye Harbour on Alderney)
let url = rjw_uktides::tide_predictions_url(&stations[0].id);
let body = ureq::get(url.as_str()).call()?.into_body();
let predictions: TidePredictions = rjw_uktides::tides_from_reader(body.into_reader())?;

// Print the times of the high and low tides
for event in predictions.tidal_event_list {
    println!("{}    {}", event.date_time, event.event_type);
}
```

### CLI example usage

The command-line tool is focused on the same use, of printing the times of high
and low tides. The following command is roughly equivalent to the predictions
part of the code shown above.

```sh
# Look up tide predictions for a known station ID, in this case Inverness in Scotland.
# Note that station IDs are not numeric; leading zeroes are important.
tides --station 0256
```

### What is EasyTide?

[EasyTide] is a public web application that allows you to look up tide
predictions for about 700 locations around Great Britain and Ireland. It uses
the following URLs which return JSON data for tide stations (a location for
which tide predictions have been made) and tide prediction data for those
stations:

- `https://easytide.admiralty.co.uk/Home/GetStations`
- `https://easytide.admiralty.co.uk/Home/GetPredictionData?stationId=...`

### Why not use the official UKHO API?

The UKHO does provide access to the same data through its "[UK Tidal API - Discovery][api]".
However, API access is subject to a "free 1-year subscription" (what happens
after?) and the terms of use include the following extremely restrictive part:

> (3.1) you may … use the data and the API for the sole purpose of developing
> web or mobile applications for use by end-users on any platform, such that
> end-users are able to access the Data
>
> (3.2) no application developed by you using the API and the Data shall enable
> any end-user to copy, reproduce, sell, store in any medium …, transmit,
> re-transmit, make or otherwise use the Materials (including but not limited
> to “caching” any Data for access by any person and/or “mirroring” any Data);

This states that I may provide "access \[to\] the Data" to "end-users", but
must not "enable any end-user to … otherwise use the Materials".

My own original purpose was to develop a small programme to run on my laptop
once per day that would send me an email with that day's tide predictions,
because I like to go for a walk along the seafront at high and low tide. My
understanding is that this is not acceptable under the Tidal API's terms of use
because the programme is not a "web or mobile application", and because I
"transmit" and "store" the tide predictions in my email.

[api]: https://developer.admiralty.co.uk/product#product=uk-tidal-api

### I'm from the UKHO and I'm unhappy about this

Perhaps take a leaf out of the Met Office's book and make your most basic tide
data API suitable for personal use without making people worry that they'll
violate the terms of use and lose access.

### Common-sense disclaimer and warning

The data from EasyTide is Crown Copyright. You should not depend on this library in
any activity that could result in harm to yourself or others. Most importantly, you
should enjoy being beside the water!

### Why is it called `rjw-uktides`?

The library name contains `uk` because the information is provided by the UK
Hydrographic Office, even though some of the tide stations are other countries.
It is not a comment about whether those countries should be part of the
political entity that is the UK.
