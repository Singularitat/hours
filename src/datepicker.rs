use std::hash::Hash;

pub use chrono::offset::{FixedOffset, Local, Utc};
use chrono::prelude::*;
pub use chrono::DateTime;
use chrono::Duration;
use eframe::egui;
use eframe::egui::{Area, DragValue, Frame, Id, Key, Order, Response, RichText, Ui, Widget};

pub struct DatePicker<'a> {
    id: Id,
    date: &'a mut NaiveDate,
}

impl<'a> DatePicker<'a> {
    /// Create new date picker with unique id and mutable reference to date.
    pub fn new<T: Hash>(id: T, date: &'a mut NaiveDate) -> Self {
        Self {
            id: Id::new(id),
            date,
        }
    }

    /// Draw names of week days as 7 columns of grid without calling `Ui::end_row`
    fn show_grid_header(&mut self, ui: &mut Ui) {
        ui.label("Mon");
        ui.label("Tue");
        ui.label("Wed");
        ui.label("Thu");
        ui.label("Fri");
        ui.label("Sat");
        ui.label("Sun");
    }

    /// Get number of days between first day of the month and Monday
    fn get_start_offset_of_calendar(&self, first_day: &NaiveDate) -> u32 {
        first_day.weekday().num_days_from_monday()
    }

    /// Get number of days between first day of the next month and Monday
    fn get_end_offset_of_calendar(&self, first_day: &NaiveDate) -> u32 {
        (7 - (first_day).weekday().num_days_from_monday()) % 7
    }

    fn show_calendar_grid(&mut self, ui: &mut Ui) {
        egui::Grid::new("calendar").show(ui, |ui| {
            self.show_grid_header(ui);
            let first_day_of_current_month = self.date.with_day(1).unwrap();
            let start_offset = self.get_start_offset_of_calendar(&first_day_of_current_month);
            let days_in_month = get_days_from_month(self.date.year(), self.date.month());
            let first_day_of_next_month =
                first_day_of_current_month + Duration::days(days_in_month);
            let end_offset = self.get_end_offset_of_calendar(&first_day_of_next_month);
            let start_date = first_day_of_current_month - Duration::days(start_offset.into());
            for i in 0..(start_offset as i64 + days_in_month + end_offset as i64) {
                if i % 7 == 0 {
                    ui.end_row();
                }
                let d = start_date + Duration::days(i);
                self.show_day_button(d, ui);
            }
        });
    }

    fn show_day_button(&mut self, date: NaiveDate, ui: &mut Ui) {
        ui.add_enabled_ui(self.date != &date, |ui| {
            ui.centered_and_justified(|ui| {
                if self.date.month() != date.month() {
                    ui.style_mut().visuals.button_frame = false;
                }
                if ui.button(date.day().to_string()).clicked() {
                    *self.date = date;
                }
            });
        });
    }

    /// Draw current month and buttons for next and previous month.
    fn show_header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.show_month_control(ui);
            self.show_year_control(ui);
            if ui.button("Today").clicked() {
                *self.date = Local::now().date_naive();
            }
        });
    }

    /// Draw button with text and add duration to current date when that button is clicked.
    fn date_step_button(&mut self, ui: &mut Ui, text: impl ToString, duration: Duration) {
        if ui.button(text.to_string()).clicked() {
            *self.date += duration;
        }
    }

    /// Draw drag value widget with current year and two buttons which substract and add 365 days to current date.
    fn show_year_control(&mut self, ui: &mut Ui) {
        self.date_step_button(ui, "<", Duration::days(-365));
        let mut drag_year = self.date.year();
        ui.add(DragValue::new(&mut drag_year));
        if drag_year != self.date.year() {
            *self.date = self.date.with_year(drag_year).unwrap();
        }
        self.date_step_button(ui, ">", Duration::days(365));
    }

    /// Draw label with current month and two buttons which substract and add 30 days to current date.
    fn show_month_control(&mut self, ui: &mut Ui) {
        self.date_step_button(ui, "<", Duration::days(-30));
        let month_string = month_to_string(self.date.month());
        ui.add(egui::Label::new(
            RichText::new(format!("{month_string: <9}")).text_style(egui::TextStyle::Monospace),
        ));
        self.date_step_button(ui, ">", Duration::days(30));
    }
}

impl<'a> Widget for DatePicker<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let formated_date = self.date.format("%Y-%m-%d");
        let button_response = ui.button(RichText::new(formated_date.to_string()).size(16.0));
        if button_response.clicked() {
            ui.memory_mut(|mem| {
                mem.toggle_popup(self.id);
            })
        }

        if ui.memory(|mem| mem.is_popup_open(self.id)) {
            let area = Area::new(self.id)
                .order(Order::Foreground)
                .default_pos(button_response.rect.left_bottom())
                .movable(false);
            let area_response = area
                .show(ui.ctx(), |ui| {
                    Frame::popup(ui.style()).show(ui, |ui| {
                        self.show_header(ui);
                        self.show_calendar_grid(ui);
                    });
                })
                .response;

            if !button_response.clicked() {
                if ui.input(|i| i.key_pressed(Key::Escape)) || area_response.clicked_elsewhere() {
                    ui.memory_mut(|mem| {
                        mem.toggle_popup(self.id);
                    });
                }
            }
        }

        button_response
    }
}

// https://stackoverflow.com/a/58188385
fn get_days_from_month(year: i32, month: u32) -> i64 {
    // let mdf = (month << 9) | (day << 4) | flags;
    NaiveDate::from_ymd_opt(
        match month {
            12 => year + 1,
            _ => year,
        },
        match month {
            12 => 1,
            _ => month + 1,
        },
        1,
    )
    .unwrap()
    .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
    .num_days()
}

#[inline]
const fn month_to_string(n: u32) -> &'static str {
    match n {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => unreachable!(),
    }
}
