#[derive(PartialEq)]
pub struct DataLine
{
    pub data: Vec<[f64; 2]>,
    pub rgb: [f32; 3],
    pub name: String,
}

impl DataLine
{
    pub fn new(name: String, xyz: Vec<[f64; 2]>) -> DataLine
    {
        DataLine
        {
            name: name,
            data: xyz,
            rgb: [255.0, 0.0, 0.0],
        }
    }
    pub fn len(&self) -> usize
    {
        return self.data.len();
    }

    pub fn clear(&mut self)
    {
        self.data.clear();
    }
    pub fn push(&mut self, element: [f64; 2])
    {
        self.data.push(element);
    }
}

#[derive(PartialEq)]
#[derive(Clone)]
pub struct DataLineDependency
{
    pub index_old: [usize; 2],
    pub index: [usize; 2],
    pub rgb: [f32; 3],
}

impl DataLineDependency
{
    pub fn new(x: usize, y: usize) -> DataLineDependency
    {
        DataLineDependency
        {
            index: [x, y],
            index_old: [x, y],
            rgb: [255.0, 0.0, 0.0],
        }
    }
}
