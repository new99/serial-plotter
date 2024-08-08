use serialport;
use std::time::{Duration, Instant};
use std::thread;
use std::str;
use std::collections::HashMap;
use std::sync::mpsc;

pub struct ReadPort {
        xyz: HashMap<i64, Vec<f64>>,
        name_port: String,
        speed: u32,
        time: u64,
        time_start: Instant,
        n: u64,
        tx: mpsc::Sender<(String, f64)>,
        run_rtx: (mpsc::Sender<bool>, mpsc::Receiver<bool>),
        error: mpsc::Sender<String>,
}


impl ReadPort {

    pub fn new(name_port: String, speed: u32, time: u64, tx: mpsc::Sender<(String, f64)>, error: mpsc::Sender<String>) -> ReadPort
    {
        ReadPort{
            name_port: name_port,
            speed: speed,
            time: time,
            tx: tx,
            n: 0,
            error: error,
            xyz: HashMap::new(),
            time_start: Instant::now(),
            run_rtx: mpsc::channel(),
        }
    }

    fn error_f(&self, s: String)
    {
        self.error.send(s).unwrap();
    }

    fn send_lost(&mut self)
    {
        let time = self.time_start.elapsed().as_millis() as f64 / 1000.0;
        self.tx.send(('t'.to_string(),time)).unwrap();

        for i in 0..self.xyz.len()
        {
            if self.xyz[&i.try_into().unwrap()].len() == 0
            {
                continue;
            }

            let x = *self.xyz[&(i as i64)].last().unwrap();
            self.tx.send(('y'.to_string()+&i.to_string(), x)).unwrap();
            self.xyz.insert(i as i64, Vec::new());
        }
    }

    fn send_mean(&mut self)
    {
        let time = self.time_start.elapsed().as_millis() as f64 / 1000.0;
        self.tx.send(('t'.to_string(),time)).unwrap();

        for i in 0..self.xyz.len()
        {
            if self.xyz[&(i as i64)].len() == 0
            {
                continue;
            }

            let mut mean:f64 = 0.0;
            let len:usize = self.xyz[&(i as i64)].len().try_into().unwrap();

            for j in &self.xyz[&(i as i64)]
            {
                mean += j;
            }

            self.tx.send(('y'.to_string()+&i.to_string(), mean/len as f64)).unwrap();
            self.xyz.insert(i as i64, Vec::new());
        }
    }

    fn send_all(&mut self)
    {

        let time = self.time_start.elapsed().as_millis() as f64 / 1000.0 + (self.time as f64 / 1000.0);

        for i in 0..self.xyz.len()
        {
            if self.xyz[&(i as i64)].len() == 0
            {
                continue;
            }

            let max = self.xyz[&(i as i64)].len() as f64;

            for j in 0..self.xyz[&(i as i64)].len()
            {
                let val = &time - (self.time as f64 / 1000.0) * (&max - j as f64) / max;
                self.tx.send(('t'.to_string(), val)).unwrap();
                self.tx.send(('y'.to_string()+&i.to_string(), self.xyz[&(i as i64)][j as usize])).unwrap();
            }
            self.xyz.insert(i as i64, Vec::new());
        }
    }

    pub fn run_tx(&self) -> mpsc::Sender<bool>
    {
        return self.run_rtx.0.clone();
    }

    pub fn read(&mut self, set: u32)
    {
        let port = serialport::new(&self.name_port, self.speed)
            .timeout(Duration::from_millis(&self.time/2))
            .open();

        if port.is_err()
        {
            let _ = &self.error_f("Failed to open port".to_string());
            return;
        }

        let mut port = port.unwrap();

        thread::sleep(Duration::from_secs(2));
        let _ = port.clear(serialport::ClearBuffer::Input);
        thread::sleep(Duration::from_secs(1));

        let mut sparkle_heart = "".to_string();
        let mut j = 0;
        let mut y = 0;

        self.time_start = Instant::now();
        self.n = 0;
        let mut iiii = 1;

        loop
        {
            let bytes = port.bytes_to_read();
            if bytes.is_err()
            {
                let _ = &self.error_f("No signal".to_string());
                return;
            }

            let mut serial_buf: Vec<u8> = vec![0; port.bytes_to_read().unwrap() as usize];

            let _ = port.read(serial_buf.as_mut_slice());

            serial_buf.retain(|&x| x != 0);
            let a = serial_buf.to_vec().clone();

            let err = str::from_utf8(&a);
            if err.is_err()
            {
                let _ = &self.error_f("Incorrect received data".to_string());
                return;
            }

            sparkle_heart += err.unwrap();

            let sparkle_heart_split: Vec<String> =  sparkle_heart.split("\r\n").map(|s| s.to_string()).collect();

            let num_n:usize = sparkle_heart_split.len();
            if num_n < 2
            {
                continue;
            }

            for i in iiii*num_n..num_n-1
            {
                if  sparkle_heart_split[i] == ""
                {
                    y += 1;
                    if y > 0
                    {
                        y = 0;
                        j = 0;
                    }
                }
                else
                {

                    let number_port = sparkle_heart_split[i].parse::<f64>();

                    if number_port.is_err()
                    {
                        let _ = &self.error_f("Warning! Incorrect received data".to_string());
                        continue;
                    }

                    let number_port = number_port.unwrap();

                    match self.xyz.get(&j) {
                        Option::None => _ = self.xyz.insert(j as i64, vec![number_port]),
                        _ => _ = self.xyz.get_mut(&j).map(|val| val.push(number_port)),
                    }

                    j += 1;
                }

            }


            if self.xyz.len() != 0
            {
                match &set {
                    1 => self.send_all(),
                    2 => self.send_lost(),
                    3 => self.send_mean(),
                    _ => todo!()
                }
            }
            sparkle_heart = sparkle_heart_split[num_n-1].clone();

            if &sparkle_heart == ""
            {
                y +=1;
            }
            for i in  self.run_rtx.1.try_iter()
            {
                if i == false
                {
                    return;
                }
            }
            iiii = 0;

            thread::sleep(Duration::from_millis(self.time));
        }
    }
}
