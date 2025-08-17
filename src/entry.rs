use std::cmp::Ordering;

pub struct Entry {
    pub date: String,
    pub description: String,
    pub start: String,
    pub end: String,
    pub hours: f64,
}

impl Entry {
    pub fn new(date: String, description: String, start: String, end: String, hours: f64) -> Self {
        Entry {
            date,
            description,
            start,
            end,
            hours,
        }
    }
}

fn get_ymd(date: &str) -> (u16, u8, u8) {
    let mut values = date.split('-');

    // It is safe to just unwrap but the entrys csv could be edited to cause a crash
    if let (Some(y), Some(m), Some(d)) = (values.next(), values.next(), values.next()) {
        return (
            y.parse().unwrap_or(0),
            m.parse().unwrap_or(0),
            d.parse().unwrap_or(0),
        );
    }

    (0, 0, 0)
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reversed because I wanted reverse ordering
        get_ymd(&other.date).cmp(&get_ymd(&self.date))
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.date == other.date
    }
}

impl Eq for Entry {}
