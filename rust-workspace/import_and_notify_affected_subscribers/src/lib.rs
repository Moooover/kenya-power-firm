use itertools::Itertools;
use location_subscription::data_transfer::{AffectedSubscriber, LocationMatchedAndLineSchedule};
use notifications::contracts::send_notification::{
    AffectedSubscriber as NotificationAffectedSubscriber, AffectedSubscriberWithLocations,
    LineWithScheduledInterruptionTime, Location,
    LocationMatchedAndLineSchedule as NotificationLocationMatchedAndLineSchedule,
};
use std::collections::HashMap;

pub fn convert_data_to_producer_input(
    data: HashMap<AffectedSubscriber, Vec<LocationMatchedAndLineSchedule>>,
) -> Vec<AffectedSubscriberWithLocations> {
    data.into_iter()
        .flat_map(|(affected_subscriber, locations)| {
            let subscriber = match affected_subscriber {
                AffectedSubscriber::DirectlyAffected(subscriber) => {
                    NotificationAffectedSubscriber::DirectlyAffected(subscriber)
                }
                AffectedSubscriber::PotentiallyAffected(subscriber) => {
                    NotificationAffectedSubscriber::PotentiallyAffected(subscriber)
                }
            };
            let split_locations = locations
                .into_iter()
                .into_group_map_by(|data| data.line_schedule.source_url.clone());

            split_locations.into_iter().map(move |(url, locations)| {
                AffectedSubscriberWithLocations {
                    source_url: url,
                    subscriber: subscriber.clone(),
                    locations: locations
                        .into_iter()
                        .map(|location| NotificationLocationMatchedAndLineSchedule {
                            line_schedule: LineWithScheduledInterruptionTime {
                                line_name: location.line_schedule.line_name,
                                from: location.line_schedule.from,
                                to: location.line_schedule.to,
                            },
                            location: Location {
                                location_id: location.location_id,
                                name: location.location_name,
                            },
                        })
                        .collect_vec(),
                }
            })
        })
        .collect_vec()
}
