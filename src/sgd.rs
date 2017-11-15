extern crate ndarray;
extern crate fnv;

use context::Context;
use ndarray_ext;
use ndarray_ext::NdArray;
use tensor;
use tensor::Tensor;


#[inline]
/// Updates shared variables with its gradients
///
/// This actually runs the computation graph.
/// For the usage, see `examples` dir in repo.
pub fn apply_gradients<T: optimizers::Optimizer>(
    variables: &[&Tensor],
    gradients: &[Tensor],
    optimizer: &mut T,
    ctx: &mut Context,
)
{
    assert_eq!(variables.len(), gradients.len());
    // run graph and get gradient arrays
    let mut grad_arrays = tensor::eval_tensors(gradients, &mut ctx.variables, &mut ctx.outputs);
    ctx.outputs.clear();
    for v in variables {
        // safe unwrap
        assert_eq!(v.op.name(), "Variable", "Can't optimize non-variable");
        let mut v_arr = ctx.variables.get_mut(v).unwrap();
        let g = maybe_reduce_grad(grad_arrays.remove(0), v_arr.shape());
        optimizer.update(v, v_arr, g);
    }
}


#[doc(hidden)]
#[inline(always)]
/// Reduces gradient's each dim by summation.
/// This is used when parameter shape and
/// gradient shape don't match due to broadcast.
pub fn maybe_reduce_grad(mut grad: NdArray, var_shape: &[usize]) -> NdArray
{
    let grad_shape = grad.shape().to_vec();
    // for each grad axis
    for (i, (g, v)) in grad_shape.iter().zip(var_shape).enumerate() {
        if g == v {
            continue; // do nothing
        } else if g < v {
            panic!("bad gradient")
        } else {
            grad = ndarray_ext::expand_dims(grad.sum_axis(ndarray::Axis(i)), i);
        }
    }
    grad
}


pub mod optimizers {
    extern crate fnv;
    use self::fnv::FnvHashMap;
    use ndarray_ext::NdArray;
    use tensor::Tensor;

    /// Trait for any stochastic gradient descent optimizer
    pub trait Optimizer {
        #[inline]
        /// Updates the variable tensor
        ///
        /// Updates `param` with `grad`
        fn update(&mut self, node: &Tensor, param: &mut NdArray, grad: NdArray);
    }


    /// Vanilla SGD optimizer
    pub struct SGD {
        pub lr: f32,
    }

    impl Optimizer for SGD {
        #[inline]
        fn update(&mut self, _: &Tensor, param: &mut NdArray, grad: NdArray)
        {
            param.scaled_add(-self.lr, &grad);
        }
    }


    /// Adam optimizer
    ///
    /// This implementation is based on http://arxiv.org/abs/1412.6980v8
    pub struct Adam {
        // static params
        pub alpha: f32,
        pub eps: f32,
        pub b1: f32,
        pub b2: f32,
        // dynamic params
        pub states: FnvHashMap<Tensor, AdamState>,
    }

    pub struct AdamState {
        m: NdArray,
        v: NdArray,
        t: f32,
    }

    impl Default for Adam {
        fn default() -> Adam
        {
            Adam {
                alpha: 0.001,
                eps: 1e-08,
                b1: 0.9,
                b2: 0.999,
                states: FnvHashMap::default(),
            }
        }
    }

    impl Optimizer for Adam {
        #[inline]
        fn update(&mut self, node: &Tensor, param: &mut NdArray, grad: NdArray)
        {
            // get current state
            let AdamState { mut m, mut v, t } = self.states.remove(&node).unwrap_or_else(|| {
                AdamState {
                    m: NdArray::zeros(param.shape()),
                    v: NdArray::zeros(param.shape()),
                    t: 1.,
                }
            });

            // make new m
            let b1 = self.b1;
            let tmp = 1. - self.b1;
            m.zip_mut_with(&grad, move |a, &g| *a = (*a) * b1 + tmp * g);
            let m_new = m;

            // make new v
            let b2 = self.b2;
            let tmp = 1. - self.b2;
            v.zip_mut_with(&grad, move |a, &g| *a = (*a) * b2 + tmp * g * g);
            let v_new = v;

            // make hat
            let mut m_hat = &m_new * (1. / (1. - b1.powf(t)));
            let v_hat = &v_new * (1. / (1. - b2.powf(t)));

            // update states
            self.states.insert(
                node.clone(),
                AdamState {
                    m: m_new,
                    v: v_new,
                    t: t + 1.,
                },
            );

            let eps = self.eps;
            m_hat.zip_mut_with(&v_hat, move |a, &b| (*a) /= b.sqrt() + eps);
            param.scaled_add(-self.alpha, &m_hat);
        }
    } // Adam end
}
