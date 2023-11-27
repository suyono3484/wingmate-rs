use std::time::Duration;
use anyhow::Context;
use time::OffsetDateTime;
use crate::init::config::{Crontab,CronTimeFieldSpec};
use crate::init::error;

const MINUTE: i8 = 0;
const HOUR: i8 = 1;
const DAY_OF_MONTH: i8 = 2;
const MONTH: i8 = 3;
const DAY_OF_WEEK: i8 = 4;

struct CronField {
    spec: CronTimeFieldSpec,
    tag: i8,
}

fn convert_cron_spec_to_vec(cron: Crontab) -> Vec<CronField> {
    let mut res: Vec<CronField> = Vec::with_capacity(5);

    res.push(CronField { spec: cron.minute, tag: MINUTE });
    res.push(CronField { spec: cron.hour, tag: HOUR });
    res.push(CronField { spec: cron.day_of_month, tag: DAY_OF_MONTH });
    res.push(CronField { spec: cron.month, tag: MONTH });
    res.push(CronField { spec: cron.day_of_week, tag: DAY_OF_WEEK });
    res
}

pub fn wait_calc(cron: Crontab, last_running: &Option<OffsetDateTime>) -> Result<Duration, error::CronConfigError> {
    let local_clock = OffsetDateTime::now_local()
        .context("getting current time in local timezone")
        .map_err(|e| { error::CronConfigError::Other { source: e } })?;

    match last_running {
        Some(t) => {
            let vec_cron = convert_cron_spec_to_vec(cron);
            for vc in vec_cron {

            }

            // match cron.minute {
            //     CronTimeFieldSpec::Any => {},
            //     CronTimeFieldSpec::Every(x) => {},
            //     CronTimeFieldSpec::Exact(x) => {},
            //     CronTimeFieldSpec::MultiOccurrence(v) => {}
            // }
            // match cron.hour {
            //     CronTimeFieldSpec::Any => {},
            //     CronTimeFieldSpec::Every(x) => {},
            //     CronTimeFieldSpec::Exact(x) => {},
            //     CronTimeFieldSpec::MultiOccurrence(v) => {}
            // }
            // match cron.day_of_month {
            //     CronTimeFieldSpec::Any => {},
            //     CronTimeFieldSpec::Every(x) => {},
            //     CronTimeFieldSpec::Exact(x) => {},
            //     CronTimeFieldSpec::MultiOccurrence(v) => {}
            // }
            // match cron.month {
            //     CronTimeFieldSpec::Any => {},
            //     CronTimeFieldSpec::Every(x) => {},
            //     CronTimeFieldSpec::Exact(x) => {},
            //     CronTimeFieldSpec::MultiOccurrence(v) => {}
            // }
            // match cron.day_of_week {
            //     CronTimeFieldSpec::Any => {},
            //     CronTimeFieldSpec::Every(x) => {},
            //     CronTimeFieldSpec::Exact(x) => {},
            //     CronTimeFieldSpec::MultiOccurrence(v) => {}
            // }
        },
        None => {
        }
    }

    Ok(Duration::from_secs(1)) //PLACEHOLDER
}