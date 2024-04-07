use orangey::OrangeyCtx;

fn main(){
    let mut rng = OrangeyCtx::new();
    for i in 0..3 {println!("({:2}) rng.rand():                   {:0<16x}", i, rng.rand())}
    rng = OrangeyCtx::new();
    for i in 0..3 {println!("({:2}) rng.rand_range(64, 128):      {}", i, rng.rand_range(64, 128))}
    rng = OrangeyCtx::new();
    for i in 0..3 {println!("({:2}) rng.uniform_double():         {}", i, rng.uniform_double())}
    rng = OrangeyCtx::new();
    for i in 0..3 {println!("({:2}) rng.all_doubles():            {}", i, rng.all_doubles())}
    rng = OrangeyCtx::new();
    for i in 0..3 {println!("({:2}) rng.gaussian():               {}", i, rng.gaussian())}
    rng = OrangeyCtx::new();
    for i in 0..3 {println!("({:2}) rng.poisson(1.33333333):      {}", i, rng.poisson(1.33333333))}
}
