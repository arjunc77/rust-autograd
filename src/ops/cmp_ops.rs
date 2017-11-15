extern crate ndarray;

use self::ndarray::Zip;
use ndarray_ext::NdArray;
use ops;
use tensor::Tensor;


pub struct Equal;
pub struct Greater {
    pub a: f32,
}
pub struct Lesser {
    pub a: f32,
}
pub struct GreaterEqual {
    pub a: f32,
}
pub struct LesserEqual {
    pub a: f32,
}


impl ops::Op for Equal {
    fn name(&self) -> &str
    {
        "Equals"
    }

    fn compute(&self, xs: &[&NdArray], _: bool) -> NdArray
    {
        let x0 = xs[0];
        let x1 = xs[1];
        assert_eq!(x0.shape(), x1.shape());

        let mut result = NdArray::zeros(x0.shape());
        Zip::from(&mut result).and(x0).and(x1).apply(|r, a, b| {
            *r = ((a == b) as i32) as f32
        });
        result
    }

    fn grad(&self, _: &Tensor, _: &[&Tensor], _: &Tensor) -> Vec<Option<Tensor>>
    {
        vec![None]
    }
}

impl ops::Op for Greater {
    fn name(&self) -> &str
    {
        "Greater"
    }

    fn compute(&self, xs: &[&::NdArray], _: bool) -> ::NdArray
    {
        xs[0].mapv(move |a| ((a > self.a) as i32) as f32)
    }

    fn grad(&self, _: &Tensor, _: &[&Tensor], _: &Tensor) -> Vec<Option<Tensor>>
    {
        vec![None]
    }
}

impl ops::Op for Lesser {
    fn name(&self) -> &str
    {
        "Lesser"
    }

    fn compute(&self, xs: &[&::NdArray], _: bool) -> ::NdArray
    {
        xs[0].mapv(move |a| ((a < self.a) as i32) as f32)
    }

    fn grad(&self, _: &Tensor, _: &[&Tensor], _: &Tensor) -> Vec<Option<Tensor>>
    {
        vec![None]
    }
}

impl ops::Op for GreaterEqual {
    fn name(&self) -> &str
    {
        "GreaterEqual"
    }

    fn compute(&self, xs: &[&::NdArray], _: bool) -> ::NdArray
    {
        xs[0].mapv(move |a| ((a >= self.a) as i32) as f32)
    }

    fn grad(&self, _: &Tensor, _: &[&Tensor], _: &Tensor) -> Vec<Option<Tensor>>
    {
        vec![None]
    }
}

impl ops::Op for LesserEqual {
    fn name(&self) -> &str
    {
        "LesserEqual"
    }

    fn compute(&self, xs: &[&::NdArray], _: bool) -> ::NdArray
    {
        xs[0].mapv(move |a| ((a <= self.a) as i32) as f32)
    }

    fn grad(&self, _: &Tensor, _: &[&Tensor], _: &Tensor) -> Vec<Option<Tensor>>
    {
        vec![None]
    }
}
