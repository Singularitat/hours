#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui_extras::{Column, TableBuilder};

use chrono::{Local, NaiveDate};
use eframe::egui::{self, FontData, FontDefinitions, IconData, RichText, Sense};
use eframe::epaint::{FontFamily, Vec2};

use datepicker::DatePicker;
use entry::Entry;
use utils::{
    parse_difference, read_archive, read_entrys, wipe_entrys, write_entry, write_to_archive,
};

mod datepicker;
mod entry;
mod utils;

struct WorkTracker {
    entrys: Vec<Entry>,
    archive: Vec<Entry>,
    viewing_archive: bool,
    total_hours: f64,
    total_hours_archive: f64,
    date: NaiveDate,
    description: String,
    start_time: String,
    end_time: String,
}

impl Default for WorkTracker {
    fn default() -> Self {
        Self {
            entrys: Vec::new(),
            archive: Vec::new(),
            viewing_archive: false,
            total_hours: 0.0,
            total_hours_archive: 0.0,
            date: Local::now().date_naive(),
            description: String::new(),
            start_time: String::new(),
            end_time: String::new(),
        }
    }
}

impl WorkTracker {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "ubuntu-light".to_owned(),
            FontData::from_static(include_bytes!("../assets/Ubuntu-Light.ttf")).into(),
        );

        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "ubuntu-light".to_owned());

        cc.egui_ctx.set_fonts(fonts);

        cc.egui_ctx.set_pixels_per_point(1.1);

        let mut entrys = Vec::new();
        let mut total_hours = 0.0;

        read_entrys(&mut entrys, &mut total_hours).ok();
        let total_hours = total_hours.clone();

        WorkTracker {
            entrys,
            total_hours,
            ..WorkTracker::default()
        }
    }

    fn top_input(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("input").show(ctx, |ui| {
            ui.add_space(10.0);

            ui.add_enabled_ui(!self.viewing_archive, |ui| {
                self.input(ui);
            });

            ui.add_space(10.0);
        });
    }

    fn input(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add(DatePicker::new("date", &mut self.date));

            ui.add_sized(
                [60.0, 18.0],
                egui::TextEdit::singleline(&mut self.start_time).hint_text("1:30pm"),
            );

            ui.label(RichText::new("-").size(18.0));

            ui.add_sized(
                [60.0, 18.0],
                egui::TextEdit::singleline(&mut self.end_time).hint_text("10:30pm"),
            );

            ui.add_sized(
                [200.0, 18.0],
                egui::TextEdit::singleline(&mut self.description).hint_text("Description"),
            );

            if ui.button("Add entry").clicked() {
                let start_time = self.start_time.to_ascii_lowercase();
                let end_time = self.end_time.to_ascii_lowercase();

                let hours = parse_difference(&start_time, &end_time);

                if let Some(hours) = hours {
                    self.total_hours += hours;

                    let date = self.date.format("%Y-%m-%d").to_string();

                    let entry =
                        Entry::new(date, self.description.clone(), start_time, end_time, hours);

                    write_entry(&entry);

                    self.entrys.push(entry);
                    self.entrys.sort();

                    self.description.clear();
                    self.start_time.clear();
                    self.end_time.clear();
                }
            }
        });
    }

    fn body(&self, body: egui_extras::TableBody, entrys: &Vec<Entry>, total_hours: f64) {
        let total_rows = entrys.len();

        if total_rows == 0 {
            return;
        }

        body.rows(18.0, total_rows + 1, |mut row| {
            let row_index = row.index();

            // Show total hours in the last row
            if row_index == total_rows {
                row.col(|_| {});
                row.col(|ui| {
                    ui.label(total_hours.to_string());
                });
            } else {
                // Is safe as we are in the range 0..entrys.len()
                let entry = unsafe { entrys.get_unchecked(row_index) };

                if entry.date.is_empty() {
                    if row_index != total_rows - 1 {
                        row.col(|_| {});
                    }
                    return;
                }

                row.col(|ui| {
                    ui.label(&entry.date);
                });
                row.col(|ui| {
                    ui.label(entry.hours.to_string());
                });
                row.col(|ui| {
                    ui.label(&entry.start);
                });
                row.col(|ui| {
                    ui.label(&entry.end);
                });
                row.col(|ui| {
                    ui.label(&entry.description);
                });
            }
        });
    }
}

impl eframe::App for WorkTracker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.top_input(ctx);

        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.add_space(7.0);

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.viewing_archive, egui::Button::new("Archive all"))
                    .clicked()
                {
                    self.entrys.sort();
                    if write_to_archive(&self.entrys).is_ok() {
                        self.entrys.clear();
                        wipe_entrys();
                    }
                };

                let text = if self.viewing_archive {
                    "Close archive"
                } else {
                    "Open archive"
                };

                if ui.button(text).clicked() {
                    self.viewing_archive = if self.viewing_archive {
                        false
                    } else {
                        if self.archive.len() == 0 {
                            read_archive(&mut self.archive, &mut self.total_hours_archive).ok();
                        }
                        true
                    }
                }
            });

            ui.add_space(3.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let table = TableBuilder::new(ui)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(75.0))
                .column(Column::exact(45.0))
                .column(Column::exact(60.0))
                .column(Column::exact(60.0))
                .column(Column::remainder());

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        let resp = ui.add(egui::Label::new("Date").sense(Sense::click()));

                        if resp.clicked() {
                            self.entrys.reverse();
                        }
                    });

                    header.col(|ui| {
                        ui.label("Hours");
                    });
                    header.col(|ui| {
                        ui.label("Start");
                    });
                    header.col(|ui| {
                        ui.label("End");
                    });
                    header.col(|ui| {
                        ui.label("Description");
                    });
                })
                .body(|body| {
                    if self.viewing_archive {
                        self.body(body, &self.archive, self.total_hours_archive);
                    } else {
                        self.body(body, &self.entrys, self.total_hours);
                    }
                });

            // ui.separator();
            // ui.horizontal(|ui| {
            //     ui.label("Total Hours");
            //     ui.add_space(6.0);
            //     ui.label(self.total_hours.to_string());
            // });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // .with_maximized(true)
            .with_icon(IconData {
                rgba: include_bytes!("..\\assets\\icon.rgba").to_vec(),
                width: 64,
                height: 64,
            })
            .with_min_inner_size(Vec2::new(800.0, 600.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        "Hours",
        options,
        Box::new(|cc| Ok(Box::new(WorkTracker::new(cc)))),
    )
}
