use strum::EnumString;

#[derive(Debug, PartialEq, EnumString, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum ButtonPrefix {
    SetMajor,
    TimetableWeekday
}
