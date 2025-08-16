use rjw_uktides::{
    Coordinates, Country, DecimalDegrees, LunarPhase, LunarPhaseType, Metres, Station, TidalEvent,
    TidalEventType, TidalHeightOccurence, TidePredictions,
};

const STATIONS_BYTES: &[u8] = include_bytes!("stations.json");
const TIDES_BYTES: &[u8] = include_bytes!("tides.json");

#[test]
fn stations() -> Result<(), rjw_uktides::Error> {
    let expected = vec![
        Station {
            id: "1603".into(),
            name: "BRAYE".to_owned(),
            country: Country::ChannelIslands,
            location: Coordinates {
                longitude: DecimalDegrees(-2.2),
                latitude: DecimalDegrees(49.716666),
            },
            continuous_heights_available: true,
        },
        Station {
            id: "0681".into(),
            name: "Gweedore Harbour".to_owned(),
            country: Country::Ireland,
            location: Coordinates {
                longitude: DecimalDegrees(-8.316666),
                latitude: DecimalDegrees(55.066666),
            },
            continuous_heights_available: true,
        },
        Station {
            id: "0102".into(),
            name: "RAMSGATE".to_owned(),
            country: Country::England,
            location: Coordinates {
                longitude: DecimalDegrees(1.416666),
                latitude: DecimalDegrees(51.333333),
            },
            continuous_heights_available: true,
        },
        Station {
            id: "0463".into(),
            name: "Connah's Quay".to_owned(),
            country: Country::Wales,
            location: Coordinates {
                longitude: DecimalDegrees(-3.05),
                latitude: DecimalDegrees(53.216666),
            },
            continuous_heights_available: false,
        },
        Station {
            id: "0627".into(),
            name: "Cranfield Point".to_owned(),
            country: Country::NorthernIreland,
            location: Coordinates {
                longitude: DecimalDegrees(-6.066666),
                latitude: DecimalDegrees(54.016666),
            },
            continuous_heights_available: true,
        },
        Station {
            id: "0297A".into(),
            name: "Gills Bay".to_owned(),
            country: Country::Scotland,
            location: Coordinates {
                longitude: DecimalDegrees(-3.166666),
                latitude: DecimalDegrees(58.633333),
            },
            continuous_heights_available: true,
        },
    ];
    let parsed = rjw_uktides::stations_from_reader(STATIONS_BYTES)?;
    for (p, e) in parsed.into_iter().zip(expected.into_iter()) {
        // Station's implementation of Eq only compares IDs
        assert_eq!(p.id, e.id);
        assert_eq!(p.name, e.name);
        assert_eq!(p.country, e.country);
        assert_eq!(p.location, e.location);
        assert_eq!(
            p.continuous_heights_available,
            e.continuous_heights_available
        );
    }

    Ok(())
}

#[test]
fn tides() -> Result<(), rjw_uktides::Error> {
    let footer_note = r#"High waters - important note. The high water duration can occur over an extended time period, i.e. a "high water stand". The predictions give the time and height of high water corresponding to the highest point."#.to_owned();
    let lunar_phase_list = vec![LunarPhase {
        date_time: jiff::civil::date(2025, 8, 23)
            .at(7, 6, 0, 0)
            .in_tz("Europe/London")
            .unwrap(),
        lunar_phase_type: LunarPhaseType::NewMoon,
    }];
    let tidal_event_list = vec![
        TidalEvent {
            date_time: jiff::civil::date(2025, 8, 17)
                .at(6, 14, 18, 0)
                .in_tz("Europe/London")
                .unwrap(),
            event_type: TidalEventType::HighWater,
            height: Metres(3.5317316090102),
            is_approximate_height: None,
            is_approximate_time: None,
        },
        TidalEvent {
            date_time: jiff::civil::date(2025, 8, 17)
                .at(11, 48, 32, 0)
                .in_tz("Europe/London")
                .unwrap(),
            event_type: TidalEventType::LowWater,
            height: Metres(1.5415957943340564),
            is_approximate_height: None,
            is_approximate_time: None,
        },
    ];
    let tidal_height_occurrence_list = vec![
        TidalHeightOccurence {
            date_time: jiff::civil::date(2025, 8, 18)
                .at(0, 0, 0, 0)
                .in_tz("Europe/London")
                .unwrap(),
            height: Metres(1.657104),
        },
        TidalHeightOccurence {
            date_time: jiff::civil::date(2025, 8, 18)
                .at(0, 30, 0, 0)
                .in_tz("Europe/London")
                .unwrap(),
            height: Metres(1.599734),
        },
    ];
    let expected = TidePredictions {
        footer_note,
        lunar_phase_list,
        tidal_event_list,
        tidal_height_occurrence_list,
    };
    let parsed = rjw_uktides::tides_from_reader(TIDES_BYTES)?;
    assert_eq!(parsed, expected);

    Ok(())
}
