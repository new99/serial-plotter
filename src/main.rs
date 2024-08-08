#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::collections::BTreeMap;
use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc;

use eframe::egui;
use egui_plot;
use egui_plot::{Line, Points, PlotPoints, Legend};
use serialport::{available_ports, SerialPortType};

use native_dialog::FileDialog;
use std::fs::File;
use std::io::{Read, Write};

mod readport;
mod dataline;
const NAME_FILE_SETTINGS: &str = "./settings.log";

fn main()-> Result<(), eframe::Error>
{

   let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.]),
    ..Default::default()
};
   return eframe::run_native(
       "serial-plotter",
       options,
       Box::new(|_cc| Ok(Box::new(MyApp::default()))),
   );
}


struct MyApp
{
    xyz: BTreeMap<usize, dataline::DataLine>,
    dependency: BTreeMap<usize, dataline::DataLineDependency>,
    get_time: f64,
    rtx: (mpsc::Sender<(String, f64)>, mpsc::Receiver<(String, f64)>),
    run_rtx: (mpsc::Sender<bool>, mpsc::Receiver<bool>),
    name_port: String,
    speed: u32,
    time: f64,
    time_start: Instant,
    run: bool,
    chart_xyz_bool: Vec<bool>,
    chart_dependency_bool: Vec<bool>,
    settings: bool,
    send: u32,
    error: (mpsc::Sender<String>, mpsc::Receiver<String>),
    error_str:String,
    info: bool,
    save_file: (bool, String),
}

impl Default for MyApp
{
    fn default() -> Self
    {
        let mut name_port = "".to_string();
        let mut speed = 9_600;
        let mut time = 1.0;
        let mut send = 1;


        let file = File::open(NAME_FILE_SETTINGS);
        if !file.is_err()
        {
            let mut contents = String::new();
            let _ = file.unwrap().read_to_string(&mut contents);
            let s = &contents.split("\n").map(|s| s.to_string()).collect::<Vec<_>>();
            name_port = s[0].clone();
            speed = s[1].parse().unwrap();
            time = s[2].parse().unwrap();
            send = s[3].parse().unwrap();
        }


        Self
        {
            xyz: BTreeMap::new(),
            dependency: BTreeMap::new(),
            get_time: 0.0,
            time_start: Instant::now(),
            rtx: mpsc::channel(),
            run_rtx: mpsc::channel(),
            run: false,
            chart_xyz_bool: Vec::new(),
            chart_dependency_bool: Vec::new(),
            settings: false,
            error: mpsc::channel(),
            error_str: "".to_string(),
            info: true,
            save_file: (false, "~/Desktop".to_string()),
            name_port: name_port,
            speed: speed,
            time: time,
            send: send,
        }


    }
}

impl eframe::App for MyApp
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {

        egui::TopBottomPanel::top("my_panel")
            .show(ctx, |ui| {
                ui.horizontal(|ui| {

                    egui::widgets::global_dark_light_mode_switch(ui);

                    if self.xyz.len() > 0 && ui.button("Info").clicked()
                    {
                        self.info = !self.info;
                    };

                    if !&self.run
                    {
                        if ui.button("Settings").clicked()
                        {
                            self.settings = !self.settings;
                        }

                        if ui.button("Start").clicked()
                        {
                            self.rtx  = mpsc::channel();
                            let sender = self.rtx.0.clone();
                            self.error  = mpsc::channel();
                            let error = self.error.0.clone();
                            self.run_rtx  = mpsc::channel();
                            self.xyz = BTreeMap::new();
                            self.dependency = BTreeMap::new();
                            self.error_str = "".to_string();

                            let mut u = readport::ReadPort::new(self.name_port.to_string(), self.speed, (self.time* 1000.0) as u64, sender, error);

                            self.run_rtx.0 = u.run_tx();
                            let t_send = self.send;

                            let _a = thread::spawn(move || {
                                u.read(t_send);
                            });

                            self.run = true;
                            self.time_start = Instant::now();
                            self.chart_xyz_bool = Vec::new();
                            self.chart_dependency_bool = Vec::new();

                            let mut file = File::create(NAME_FILE_SETTINGS).unwrap();
                            let _ = file.write_all(&self.name_port.as_bytes());
                            let _ = file.write_all(b"\n");
                            let _ = file.write_all(&self.speed.to_string().as_bytes());
                            let _ = file.write_all(b"\n");
                            let _ = file.write_all(&self.time.to_string().as_bytes());
                            let _ = file.write_all(b"\n");
                            let _ = file.write_all(&self.send.to_string().as_bytes());
                            let _ = file.write_all(b"\n");
                        };
                    }
                    else
                    {
                        if ui.button("Stop").clicked()
                        {
                            let _ = self.run_rtx.0.send(false);
                            self.run = false;
                        };
                    }

                    if self.xyz.len() > 1 && ui.button("Add dependency").clicked()
                    {
                        self.dependency.insert(self.dependency.len(),  dataline::DataLineDependency::new(0, 1));

                        if self.chart_dependency_bool.len() < self.dependency.len()
                        {
                            self.chart_dependency_bool = vec![true; self.dependency.len()];
                        }
                    }

                    if self.xyz.len() > 0
                    {
                         ui.label("Total time: ".to_string() + &(self.get_time as u64 / 60 / 60 ).to_string() + ":" + &(self.get_time as u64 / 60 % 60 ).to_string() + ":" + &(self.get_time as u64 % 60 ).to_string());
                    }
                });
            });



        if self.info
        {
            egui::SidePanel::left("list_plot_panel")
                .resizable(true)
                .width_range(0.0..=2000.0)
                .show(ctx, |ui| {
                if &self.error_str != ""
                {
                    ui.label(egui::RichText::new(&self.error_str).color(egui::Color32::RED));
                }

                if self.run
                {
                    for i in  self.error.1.try_iter()
                    {
                        self.error_str = i.to_string();
                        if self.error_str.find("Warning!").is_some()
                        {
                            continue;
                        }
                        self.run = false;
                        return;
                    }


                    let send = match &self.send {
                        1 => "all",
                        2 => "lost",
                        3 => "mean",
                        _ => "error"
                    };

                    ui.label("Port: ".to_string() + &self.name_port.to_string() + " time: " + &self.time.to_string() + "s, send: " + &send.to_string());

                    if ui.button("Reset").clicked()
                    {
                        for i in 0..self.xyz.len()
                        {
                            self.xyz.get_mut(&i).expect("").clear();
                        }
                    }
                }


                if self.settings && !self.run
                {
                    ui.horizontal(|ui|
                    {
                        ui.label("Port:");
                        egui::ComboBox::from_label("")
                            .selected_text(&self.name_port)
                            .show_ui(ui, |ui| {

                                let port = available_ports().expect("");
                                for p in &port
                                {
                                    ui.selectable_value(&mut self.name_port, p.port_name.to_string(),  &p.port_name.to_string());
                                };
                            });


                        let mut tmp_value = self.speed.to_string();
                        ui.label("speed:");
                        let _ = ui.add(egui::TextEdit::singleline(&mut tmp_value).clip_text(false).desired_width(ui.available_width()/3.0));
                        if tmp_value == ""
                        {
                            self.speed = 0;
                        }
                        else
                        {
                            self.speed = tmp_value.parse().unwrap()
                        }

                    });

                    ui.collapsing("Properities port", |ui| {
                        let port = available_ports().expect("");

                        let p = port.iter().find(|&x| x.port_name.to_string() == self.name_port);

                        if p == None
                        {
                            ui.label("Failed to open port".to_string());
                            return;
                        }

                        let p = p.unwrap();

                        match &p.port_type {
                            SerialPortType::UsbPort(info) => {
                                ui.label("Type: USB");
                                ui.label("VID: ".to_string() + &info.vid.to_string() + " PID: " + &info.pid.to_string());
                                ui.label("Serial Number: ".to_string() +  info.serial_number.as_ref().map_or("", String::as_str));
                                ui.label("Manufacturer: ".to_string() + info.manufacturer.as_ref().map_or("", String::as_str));
                                ui.label("Product: ".to_string() + info.product.as_ref().map_or("", String::as_str));
                                #[cfg(feature = "usbportinfo-interface")]
                                ui.label("Interface: ".to_string() + info.interface.as_ref().map_or("".to_string(), |x| format!("{:02x}", *x)));
                            }
                            SerialPortType::BluetoothPort => {
                                ui.label("Type: Bluetooth");
                            }
                            SerialPortType::PciPort => {
                                ui.label("Type: PCI");
                            }
                            SerialPortType::Unknown => {
                                ui.label("Type: Unknown");
                            }
                        }

                    });

                    match &self.send {
                        2 => ui.add(egui::DragValue::new(&mut self.time).range(0.002..=60.0).prefix("Time, s: ")),
                        _ => ui.add(egui::DragValue::new(&mut self.time).range(0.1..=60.0).prefix("Time, s: "))
                    };


                    ui.horizontal(|ui| {
                        ui.label("Take:");
                        ui.radio_value(&mut self.send, 1, "all");
                        ui.radio_value(&mut self.send, 2, "lost");
                        ui.radio_value(&mut self.send, 3, "mean");
                    });

                    ui.horizontal(|ui| {
                        if self.xyz.len() > 0 && ui.button("Save").clicked()
                        {
                            let save_file = FileDialog::new().set_location(&self.save_file.1).show_save_single_file().unwrap();

                            if save_file == Option::None
                            {
                                return;
                            }
                            self.save_file.1 = save_file.expect("error save file").into_os_string().into_string().unwrap();

                            let mut file = File::create(&self.save_file.1).unwrap();

                            let _ = file.write_all(b"t,s\t");


                            for i in 0..self.xyz.len()
                            {
                                let red = self.xyz.get(&i).expect("").name.to_string() + "\t";

                                let _ = file.write_all(red.as_bytes());
                            };
                            let _ = file.write_all(b"\n");

                            let max = self.xyz.iter().min_by(|a, b| a.1.len().cmp(&b.1.len())).unwrap().1.len();


                            for j in 0..max
                            {
                                let mut red = self.xyz[&(0 as usize)].data[j as usize][0].to_string() + "\t";
                                for i in 0..self.xyz.len()
                                {
                                    if &self.xyz[&(i as usize)].len() > &(j as usize)
                                    {
                                        red += &(self.xyz[&(i as usize)].data[j as usize][1].to_string() + "\t");
                                    }
                                }
                                let _ = file.write_all(red.as_bytes());
                                let _ = file.write_all(b"\n");
                            }
                        }
                    });
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(egui::Separator::default().shrink(0.0));
                    for i in 0..self.xyz.len()
                    {
                        ui.horizontal(|ui| {
                                ui.add(egui::Checkbox::new(&mut self.chart_xyz_bool[i], ""));
                            let _ = ui.add(egui::TextEdit::singleline(&mut self.xyz.get_mut(&i).unwrap().name).clip_text(false).desired_width(ui.available_width()/3.0));
                            let _ = egui::widgets::color_picker::color_edit_button_rgb(ui, &mut self.xyz.get_mut(&i).unwrap().rgb);
                        });

                    }

                    ui.add(egui::Separator::default().shrink(0.0));
                    let mut rem_i = false;
                    for i in 0..self.dependency.len()
                    {
                        if rem_i
                        {
                            return;
                        }
                        ui.horizontal(|ui| {
                            ui.add(egui::Checkbox::new(&mut self.chart_dependency_bool[i], ""));
                            egui::ComboBox::from_id_source(i*2)
                                .selected_text(format!("{:?}", &self.xyz.get(&self.dependency.get_mut(&i).expect(&i.to_string()).index[0]).unwrap().name.to_string()))
                                .show_ui(ui, |ui| {
                                    for p in 0..self.xyz.len()
                                    {
                                        ui.selectable_value(&mut self.dependency.get_mut(&i).unwrap().index[0], p,  &self.xyz.get(&p).unwrap().name.to_string());

                                    };
                                });

                            egui::ComboBox::from_id_source(i*2+1)
                                .selected_text(format!("{:?}", &self.xyz.get(&self.dependency.get_mut(&i).unwrap().index[1]).unwrap().name.to_string()))
                                .show_ui(ui, |ui| {
                                    for p in 0..self.xyz.len()
                                    {
                                        ui.selectable_value(&mut self.dependency.get_mut(&i).unwrap().index[1], p,  &self.xyz.get(&p).unwrap().name.to_string());

                                    };
                                });

                            let _ = egui::widgets::color_picker::color_edit_button_rgb(ui, &mut self.dependency.get_mut(&i).unwrap().rgb);

                            if ui.button("Remove").clicked()
                            {
                                rem_i = true;
                                for j in i..self.dependency.len()-1
                                {
                                    self.chart_dependency_bool[j] = self.chart_dependency_bool[j+1];
                                    self.dependency.insert(j, self.dependency.get(&(&j+1)).unwrap().clone());
                                }
                                self.chart_dependency_bool.pop();
                                self.dependency.remove_entry(&(&self.dependency.len()-1));
                                return;
                            }
                        });
                    }
                });
            });
        }

        if self.dependency.len() > 0
        {
            egui::SidePanel::right("chart_dependency")
                .resizable(true)
                .width_range(0.0..=2000.0)
                .show(ctx, |ui| {
                    let mut plot_show:f32 = 0.0;

                    for i in &self.chart_dependency_bool
                    {
                        plot_show += *i as u8 as f32;
                    }
                    let plot_height = ui.available_height()/plot_show;

                    for (i, dependency)  in self.dependency.iter_mut()
                    {
                        if self.chart_dependency_bool[*i]
                        {

                            let index_x = dependency.index[0];
                            let index_y = dependency.index[1];

                            let size = if self.xyz.get(&index_x).expect("error plot").len() < self.xyz.get(&index_y).expect("error plot").len() { self.xyz.get(&index_x).expect("error plot").len() } else { self.xyz.get(&index_y).expect("error plot").len()};
                            let line_points: PlotPoints = (0..size).map(|i| {
                                                                    let x = self.xyz.get(&index_x).expect("error plot").data[i][1];
                                                                    let y = self.xyz.get(&index_y).expect("error plot").data[i][1];
                                                                    [x, y]
                                                                }).collect();

                            let name_line = self.xyz.get(&index_y).unwrap().name.to_string() + &"(".to_string() + &self.xyz.get(&index_x).unwrap().name.to_string() + ")";

                            let line = Points::new(line_points).name(&name_line).color(egui::Color32::from_rgb((dependency.rgb[0] * 255.0) as u8, (dependency.rgb[1] * 255.0) as u8, (dependency.rgb[2] * 255.0) as u8));

                            egui_plot::Plot::new("plot ".to_string() + &name_line + &i.to_string())
                                .legend(Legend::default())
                                .height(plot_height)
                                .clamp_grid(true)
                                .auto_bounds([true, true].into())
                                .show(ui, |plot_ui| plot_ui.points(line))
                                .response;
                         };
                    }
                });
        }

        egui::CentralPanel::default()
            .show(ctx, |ui| {
                if self.run
                {
                    for y in  self.rtx.1.try_iter()
                    {
                        match y.0.chars().collect::<Vec<_>>()[0] {
                            't' => self.get_time = y.1,
                            'y' => (|| {

                                    let index = (y.0.chars().collect::<Vec<_>>()[1] as i32 - 0x30)  as usize;
                                    match self.xyz.get(&(index)) {
                                        Option::None => _ = self.xyz.insert(index, dataline::DataLine::new(index.to_string(), vec![[self.get_time, y.1]])),
                                        _ => _ = self.xyz.get_mut(&(index)).unwrap().push([self.get_time, y.1]),
                                    }
                                })(),
                            _ => todo!(),
                        };
                    };
                    ctx.request_repaint_after(Duration::from_millis((&self.time*1000.0) as u64));
                };

                if self.xyz.len()>0
                {
                    if self.chart_xyz_bool.len() < self.xyz.len()
                    {
                        self.chart_xyz_bool = vec![true; self.xyz.len()];
                    }

                    let mut plot_show:f32 = 0.0;

                    for i in &self.chart_xyz_bool
                    {
                        plot_show += *i as u8 as f32;
                    }

                    let plot_height = ui.available_height()/plot_show;

                    for (i,xyz) in self.xyz.iter()
                    {
                        if self.chart_xyz_bool[*i]
                        {
                            let line_points: PlotPoints = PlotPoints::new(xyz.data.to_vec().into_iter().collect());
                            let name_line = &xyz.name;

                            let line = Line::new(line_points).name(&name_line).color(egui::Color32::from_rgb((xyz.rgb[0] * 255.0) as u8, (xyz.rgb[1] * 255.0) as u8, (xyz.rgb[2] * 255.0) as u8));

                            egui_plot::Plot::new("plot ".to_string() + &name_line)
                                .legend(Legend::default())
                                .height(plot_height)
                                .width(ui.available_width())
                                .clamp_grid(true)
                                .auto_bounds([true, true].into())
                                .show(ui, |plot_ui| {plot_ui.line(line)})
                                .response;
                         };
                    };
                };
            });
    }
}
